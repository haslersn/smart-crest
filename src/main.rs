#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use env_logger::Builder;
use env_logger::Env;
use pcsc::Attribute;
use pcsc::Card;
use pcsc::Context;
use pcsc::Protocols;
use pcsc::ReaderState;
use pcsc::Scope;
use pcsc::ShareMode;
use pcsc::State;
use pcsc::PNP_NOTIFICATION;
use reqwest::Client;
use std::collections::VecDeque;
use std::error::Error;
use std::ffi::CStr;

pub type Result<T = ()> = std::result::Result<T, Box<Error>>;

struct CardReader {
    context: Context,
    reader_states: Vec<ReaderState>,
    responses: VecDeque<Vec<u8>>,
}

impl CardReader {
    pub fn new() -> Result<Self> {
        Ok(Self {
            context: Context::establish(Scope::User)?,
            reader_states: vec![
                // Listen for reader insertions/removals, if supported
                // See https://docs.rs/pcsc/2.1.1/pcsc/struct.Context.html#method.get_status_change
                ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
            ],
            responses: VecDeque::new(),
        })
    }

    fn await_response(&mut self) -> Result<Vec<u8>> {
        loop {
            match self.responses.pop_front() {
                Some(resp) => {
                    return Ok(resp);
                }
                None => {
                    self.await_update()?;
                }
            }
        }
    }

    fn await_update(&mut self) -> Result {
        // Add new readers
        let mut name_buf = vec![0; self.context.list_readers_len()?];
        let reader_names = self.context.list_readers(&mut name_buf)?;
        for name in reader_names {
            if !self.reader_states.iter().any(|rs| rs.name() == name) {
                info!("Adding reader {:?}", name);
                let state = ReaderState::new(name, State::UNAWARE);
                self.reader_states.push(state);
            }
        }

        // Wait until a reader state changes
        self.context
            .get_status_change(None, &mut self.reader_states)?;

        // Update the changed reader states
        for rs in &mut self.reader_states {
            rs.sync_current_state();
        }

        // Remove dead readers
        self.reader_states.retain(|rs| {
            if rs.event_state().intersects(State::UNKNOWN | State::IGNORE) {
                info!("Removing reader {:?}", rs.name());
                false
            } else {
                true
            }
        });

        // Read from the readers whose reader state changed
        self.read_changed()?;

        Ok(())
    }

    fn read_changed(&mut self) -> Result {
        for rs in &self.reader_states {
            info!("{:?} {:?} {:?}", rs.name(), rs.event_state(), rs.atr());
            if rs.event_state().contains(State::CHANGED | State::PRESENT) {
                match self.read_card(rs.name()) {
                    Ok(resp) => self.responses.push_back(resp),
                    Err(err) => error!("Error reading card: {}", err),
                }
            }
        }
        Ok(())
    }

    fn read_card(&self, name: &CStr) -> Result<Vec<u8>> {
        // Connect to the card
        let card = self
            .context
            .connect(name, ShareMode::Shared, Protocols::ANY)?;

        // TODO: Use this
        let _ = Self::get_card_attribute(&card, Attribute::AtrString);

        // Send an APDU command
        Self::transmit_adpu(&card, b"\xFF\xCA\x00\x00\x00")
    }

    fn get_card_attribute(card: &Card, attr: Attribute) -> Result<Vec<u8>> {
        info!("Getting attribute: {:?}", attr);
        let mut resp_buf = [0; pcsc::MAX_BUFFER_SIZE];
        let resp = card.get_attribute(attr, &mut resp_buf)?;
        info!("Response: {:?}", hex::encode(resp));
        Ok(resp.to_vec())
    }

    fn transmit_adpu(card: &Card, command: &[u8]) -> Result<Vec<u8>> {
        info!("APDU transmission: {:?}", hex::encode(command));
        let mut resp_buf = [0; pcsc::MAX_BUFFER_SIZE];
        let resp = card.transmit(command, &mut resp_buf)?;
        info!("Response: {:?}", hex::encode(resp));
        Ok(resp.to_vec())
    }
}

impl Iterator for CardReader {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Result<Vec<u8>>> {
        Some(self.await_response())
    }
}

#[derive(Deserialize)]
struct Config {
    endpoint: String,
}

fn main() -> Result {
    // initialize logger w/ log level "info"
    Builder::from_env(Env::new().default_filter_or("info")).init();

    let conf = read_config()?;
    let rest_client = Client::new();

    for resp in CardReader::new()? {
        match resp {
            Ok(resp) => handle_response(&conf, &rest_client, resp),
            Err(err) => error!("Got error instead of response: {:?}", err),
        }
    }

    Ok(())
}

fn read_config() -> Result<Config> {
    let conf_str = std::fs::read_to_string("smart_crest.toml")?;
    toml::from_str(&conf_str).map_err(From::from)
}

fn handle_response(conf: &Config, rest_client: &Client, resp: Vec<u8>) {
    let token = hex::encode(&resp[0..resp.len() - 2]);
    let url = conf.endpoint.clone().replace("{}", &token);
    let _ = rest_client
        .post(&url)
        .send()
        .map_err(|err| error!("HTTP request failed: {:?}", err));
}

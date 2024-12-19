use near_sdk::collections::UnorderedSet;
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    env, log, near_bindgen, require,
    serde::{Deserialize, Serialize},
    AccountId, BorshStorageKey, Gas, GasWeight, PanicOnDefault, PromiseOrValue,
};
use schemars::JsonSchema;
use std::convert::TryInto;

const MIN_REQUEST_GAS: Gas = Gas::from_tgas(40);
const MIN_RESPONSE_GAS: Gas = Gas::from_tgas(40);
const DATA_ID_REGISTER: u64 = 0;
mod events;
mod utils;

use crate::utils::*;

pub type CryptoHash = [u8; 32];

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, JsonSchema, Clone)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Request {
    data_id: CryptoHash,
    #[schemars(with = "String")]
    originator_id: AccountId,
    message: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, JsonSchema, Clone)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Response {
    pub ok: bool,
    pub data: Option<String>,
    pub signature: Option<String>,
}

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize))]
#[serde(crate = "near_sdk::serde")]
pub struct ResponseMsg {
    current_champion: String,
    guess_wins: bool,
    reason: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, JsonSchema, Clone)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct AgentData {
    request: Request,
    champions: Vec<String>,
    prompt: String,
}

pub type RequestId = u64;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    agent_name: String,
    agent_public_key: String,
    agent_system_prompt: String,

    paused: bool,
    requests: UnorderedMap<RequestId, Request>,
    responses: LookupMap<RequestId, Response>,
    num_requests: u64,

    owner_id: AccountId,
    operator_id: AccountId,

    current_champion: String,
    champion_owner: AccountId,
    all_champions: UnorderedSet<String>,
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    Requests,
    Responses,
    AllChampions,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        operator_id: AccountId,

        initial_champion: String,

        agent_name: String,
        agent_system_prompt: String,
        agent_public_key: String,
    ) -> Self {
        let mut all_champions = UnorderedSet::new(StorageKey::AllChampions);
        all_champions.insert(&initial_champion);

        Self {
            agent_name,
            agent_public_key,
            agent_system_prompt,

            owner_id: owner_id.clone(),
            operator_id,

            paused: false,

            requests: UnorderedMap::new(StorageKey::Requests),
            responses: LookupMap::new(StorageKey::Responses),
            num_requests: 0,

            current_champion: initial_champion.to_string(),
            champion_owner: owner_id,
            all_champions,
        }
    }

    pub fn get_all_champions(&self) -> Vec<String> {
        self.all_champions.to_vec()
    }

    pub fn get_champion(&self) -> String {
        self.current_champion.clone()
    }

    pub fn get_champion_owner(&self) -> AccountId {
        self.champion_owner.clone()
    }

    pub fn get_request(&self, request_id: RequestId) -> Request {
        self.requests.get(&request_id).unwrap()
    }

    pub fn get_question(&self) -> String {
        format!("What beats {}?", self.current_champion)
    }

    pub fn get_requests(&self) -> Vec<(RequestId, Request)> {
        self.requests.iter().collect()
    }

    pub fn agent_data(&self, request_id: RequestId) -> AgentData {
        AgentData {
            request: self.get_request(request_id),
            champions: self.get_all_champions(),
            prompt: self.agent_system_prompt.clone(),
        }
    }

    pub fn set_system_prompt(&mut self, prompt: String) {
        self.assert_operator();
        self.agent_system_prompt = prompt;
    }

    pub fn request(&mut self, message: String) {
        self.assert_paused();

        require!(
            remaining_gas() >= MIN_REQUEST_GAS,
            "Not enough remaining gas to make the request"
        );

        let message = message.to_lowercase();
        assert!(is_valid_string(message.as_str()), "Illegal input string");

        let account_id: AccountId = env::predecessor_account_id();
        let request_id: RequestId = self.num_requests;

        let yield_promise = env::promise_yield_create(
            "await_response",
            &serde_json::to_vec(&(request_id,)).unwrap(),
            MIN_RESPONSE_GAS,
            GasWeight(0),
            DATA_ID_REGISTER,
        );

        let data_id: CryptoHash = env::read_register(DATA_ID_REGISTER)
            .expect("")
            .try_into()
            .expect("");

        let request_with_data_id = Request {
            data_id,
            originator_id: account_id.clone(),
            message: message.clone(),
        };

        self.requests.insert(&request_id, &request_with_data_id);
        self.num_requests += 1;

        events::emit::run_agent(&self.agent_name, &message, Some(request_id));

        env::promise_return(yield_promise);
    }

    pub fn respond(&mut self, data_id: CryptoHash, request_id: RequestId, response: Response) {
        self.assert_operator();

        if self.requests.get(&request_id).is_none() {
            panic!("Request ID not found");
        }

        self.responses.insert(&request_id, &response);

        env::promise_yield_resume(&data_id, &serde_json::to_vec(&(request_id,)).unwrap());
    }

    #[private]
    pub fn await_response(&mut self, request_id: RequestId) -> PromiseOrValue<Response> {
        let response: Option<Response> = self.responses.get(&request_id);
        if let Some(response) = response {
            self.responses.remove(&request_id);

            let request = self.requests.remove(&request_id).expect("Wrong request");

            let response_text = response.data.clone().unwrap_or_default();

            let parsed_message = serde_json::from_str::<ResponseMsg>(&response_text)
                .expect("Wrong response message format");

            assert_eq!(
                parsed_message.current_champion, self.current_champion,
                "Illegal current champion"
            );

            if response.ok && parsed_message.guess_wins {
                self.set_champion(
                    request.message.to_lowercase(),
                    request.originator_id.clone(),
                );
                log!(
                    "Player {} won: {}",
                    request.originator_id.clone(),
                    parsed_message.reason
                );
            } else {
                log!(
                    "Player {} lost: {}",
                    request.originator_id.clone(),
                    parsed_message.reason
                );
            }

            PromiseOrValue::Value(response)
        } else {
            panic!("Response is missing for {}", request_id);
        }
    }

    pub fn remove_request(&mut self, request_id: RequestId) {
        self.assert_operator();
        self.requests.remove(&request_id);
        self.responses.remove(&request_id);
    }
}

impl Contract {
    fn set_champion(&mut self, new_champion: String, new_champion_owner: AccountId) {
        self.all_champions.insert(&new_champion);
        self.current_champion = new_champion;
        self.champion_owner = new_champion_owner;
    }
}

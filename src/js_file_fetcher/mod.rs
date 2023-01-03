use js_sys::Uint8Array;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::RequestInit;
pub mod request_utils;

pub enum FileRequestState {
    Queued,
    InProgress,
    Finished,
    Failed,
}

#[wasm_bindgen]
pub struct FileRequest {
    uuid: Uuid,
    req_init: web_sys::RequestInit,
    req_url: String,
}

impl FileRequest {
    pub fn new(uuid: Uuid, req_init: web_sys::RequestInit, req_url: String) -> Self {
        Self {
            uuid,
            req_init,
            req_url,
        }
    }
    pub fn get_uuid(&self) -> Uuid {
        self.uuid.clone()
    }
}

#[wasm_bindgen]
impl FileRequest {
    pub fn get_uuid_u8(&self) -> Uint8Array {
        let uuid_byte_ref: &[u8] = &self.uuid.to_bytes_le();
        js_sys::Uint8Array::from(uuid_byte_ref)
    }

    pub fn get_request_init(&self) -> RequestInit {
        self.req_init.clone()
    }

    pub fn get_request_url(&self) -> String {
        self.req_url.clone()
    }
}

#[wasm_bindgen]
pub struct FileFetcher {
    queued_requests: std::collections::HashMap<Uuid, FileRequest>,
    queued_uuid: std::collections::VecDeque<Uuid>,
    finished_requests: RefCell<std::collections::HashMap<Uuid, js_sys::Uint8Array>>,
    failed_requests: RefCell<std::collections::HashMap<Uuid, JsValue>>,
}

impl FileFetcher {
    pub fn new() -> Self {
        Self {
            queued_requests: HashMap::new(),
            queued_uuid: VecDeque::new(),
            finished_requests: RefCell::new(HashMap::new()),
            failed_requests: RefCell::new(HashMap::new()),
        }
    }

    pub fn request_file(
        &mut self,
        id: Uuid,
        request_init: web_sys::RequestInit,
        request_url: &str,
    ) {
        let file_req = FileRequest::new(id, request_init, request_url.into());
        self.queued_uuid.push_back(id);
        self.queued_requests.insert(id, file_req);
    }

    pub fn get_request_state(&self, uuid: &Uuid) -> FileRequestState {
        if self.finished_requests.borrow().contains_key(uuid) {
            FileRequestState::Finished
        } else if self.failed_requests.borrow().contains_key(uuid) {
            FileRequestState::Failed
        } else if self.queued_requests.contains_key(uuid) {
            FileRequestState::Queued
        } else {
            FileRequestState::InProgress
        }
    }

    pub fn take_finished_request(&self, uuid: &Uuid) -> Option<js_sys::Uint8Array> {
        self.finished_requests.borrow_mut().remove(uuid)
    }

    pub fn take_failed_request(&self, uuid: &Uuid) -> Option<JsValue> {
        self.failed_requests.borrow_mut().remove(uuid)
    }
}

#[wasm_bindgen]
impl FileFetcher {
    pub fn get_next_request(&mut self) -> Option<FileRequest> {
        match self.queued_uuid.pop_front() {
            Some(id) => match self.queued_requests.remove(&id) {
                Some(f_request) => Some(f_request),
                None => None,
            },
            None => None,
        }
    }

    pub fn push_finish_request(&mut self, request: FileRequest, buffer: js_sys::Uint8Array) {
        self.finished_requests.borrow_mut().insert(request.uuid, buffer);
    }

    pub fn push_failed_request(&mut self, request: FileRequest, js_error: JsValue) {
        self.failed_requests.borrow_mut().insert(request.uuid, js_error);
    }
}
pub enum FileFetchRequest {
    InQueue(Uuid),
    Data(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum FetchError {
    ImpossibleToTakeFile,
    RequestFailed(JsValue),
    ImpossibletoTakeError,
}

impl FileFetchRequest {
    pub fn is_data(&self) -> bool {
        match self {
            FileFetchRequest::Data(_) => true,
            FileFetchRequest::InQueue(_) => false,
        }
    }
    pub fn is_queue(&self) -> bool {
        match self {
            FileFetchRequest::Data(_) => false,
            FileFetchRequest::InQueue(_) => true,
        }
    }
    pub fn try_transform(&mut self, file_fetcher: &FileFetcher) -> Result<(), FetchError> {
        match self {
            FileFetchRequest::InQueue(id) => match file_fetcher.get_request_state(&id) {
                FileRequestState::Queued => Ok(()),
                FileRequestState::InProgress => Ok(()),
                FileRequestState::Finished => match file_fetcher.take_finished_request(&id) {
                    Some(data) => {
                        *self = FileFetchRequest::Data(data.to_vec());
                        Ok(())
                    }
                    None => Err(FetchError::ImpossibleToTakeFile),
                },
                FileRequestState::Failed => match file_fetcher.take_failed_request(&id) {
                    Some(failed_request) => Err(FetchError::RequestFailed(failed_request)),
                    None => Err(FetchError::ImpossibletoTakeError),
                },
            },
            FileFetchRequest::Data(_) => Ok(()),
        }
    }
}

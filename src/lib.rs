use actix_web::dev::{ServiceRequest, ServiceResponse};

pub trait Extension {
    /// Determines the order in which extensions are applied. Lower the number, the earlier the
    /// extension is applied.
    fn order(&self) -> usize {
        usize::MAX
    }

    /// Determines if the request should continue to be processed.
    ///
    /// Used in authorization/authentication scenarios.
    fn guard(&self, req: &ServiceRequest) -> bool {
        true
    }

    /// Invoked when the request is processed. Can be used for logging, etc.
    fn event(&self, req: &ServiceRequest, res: &ServiceResponse) {}

    /// Modifies the request. Can be used to add/remove HTTP Headers, modify the request body, etc.
    fn modify_request(&self, req: &mut ServiceRequest) {}

    /// Modifies the response. Can be used to add/remove HTTP Headers, modify the response body, etc.
    fn modify_response(&self, res: &mut ServiceResponse) {}
}

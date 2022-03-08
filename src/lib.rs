use actix_web::dev::{ServiceRequest, ServiceResponse};

pub trait Extension {
    fn order(&self) -> usize {
        usize::MAX
    }
    fn guard(&self, req: &ServiceRequest) -> bool {
        true
    }
    fn before_request(&self, req: &mut ServiceRequest) {}
    fn after_response(&self, res: &mut ServiceResponse) {}
}

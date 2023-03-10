use crate::middleware::MiddlewareStatus::Continue;
use crate::middleware::MiddlewareStatus::Completed;
use crate::transport_request::TransportRequest;
use crate::transport_response::TransportResponse;

#[derive(PartialEq)]
pub enum MiddlewareStatus {
    Completed(TransportRequest, TransportResponse),
    Continue(TransportRequest, TransportResponse),
}

pub trait Middleware {
    fn process(&self, middleware_status: MiddlewareStatus) -> Result<MiddlewareStatus, ()>;

    fn priority(&self) -> u16;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct S1;

    struct S2;

    struct S3;

    impl Middleware for S1 {
        fn process(&self, middleware_status: MiddlewareStatus) -> Result<MiddlewareStatus, ()> {
            println!("s1");
            Ok(middleware_status)
        }

        fn priority(&self) -> u16 {
            todo!()
        }
    }

    impl Middleware for S2 {
        fn process(&self, middleware_status: MiddlewareStatus) -> Result<MiddlewareStatus, ()> {
            println!("s2");
            return match middleware_status {
                Completed(req, resp) => Ok(Completed(req, resp)),
                Continue(req, resp) => Ok(Completed(req, resp))
            };
        }

        fn priority(&self) -> u16 {
            todo!()
        }
    }

    impl Middleware for S3 {
        fn process(&self, middleware_status: MiddlewareStatus) -> Result<MiddlewareStatus, ()> {
            println!("s3");
            Ok(middleware_status)
        }

        fn priority(&self) -> u16 {
            todo!()
        }
    }

    #[test]
    fn this_is_test() {
        let vector: Vec<Box<dyn Middleware>> = vec![Box::new(S1), Box::new(S2), Box::new(S3)];

        test_call(&vector);
    }


    fn test_call(middlewares: &Vec<Box<dyn Middleware>>) -> MiddlewareStatus {
        let mut status = Continue(TransportRequest::default(), TransportResponse::default());

        let mut index = 0;
        for m in middlewares {
            status = m.process(status).unwrap();
            match status {
                Completed(_, _) => { break; }
                Continue(_, _) => {}
            }
            index += 1;
        }

        for i in (0..index).rev() {
            status = middlewares[i].process(status).unwrap();
        }

        return status;
    }
}

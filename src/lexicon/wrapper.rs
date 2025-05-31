use crate::lexicon::client::{AtpServiceClient, Service};
use std::sync::Arc;
use atrium_xrpc::XrpcClient;

// Implement new for Service
impl<T> Service<T>
where
    T: XrpcClient + Send + Sync + 'static,
{
    pub fn new(xrpc: Arc<T>) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct ServiceWrapper<T>
where
    T: XrpcClient + Send + Sync + 'static,
{
    inner: Service<T>,
}

impl<T> ServiceWrapper<T>
where
    T: XrpcClient + Send + Sync + 'static,
{
    pub fn new(xrpc: Arc<T>) -> Self {
        Self {
            inner: Service::new(xrpc),
        }
    }
}

pub struct AtpServiceClientWrapper<T>
where
    T: XrpcClient + Send + Sync + 'static,
{
    inner: AtpServiceClient<T>,
}

impl<T> AtpServiceClientWrapper<T>
where
    T: XrpcClient + Send + Sync + 'static,
{
    pub fn new(xrpc: T) -> Self {
        let service = ServiceWrapper::new(Arc::new(xrpc)).inner;
        Self {
            inner: AtpServiceClient { service },
        }
    }
} 
//! Set base URI of requests.
use core::fmt;
use std::{
    hash::Hash,
    ops::Deref,
    pin::{self, Pin},
    sync::Arc,
    task::{Context, Poll},
};

use futures::{future::BoxFuture, Future};
use http::{uri, Method, Request, Response};
use hyper::body::Incoming;
use kube::{
    client::{Body, DynBody},
    runtime::reflector::{ObjectRef, Store},
    Resource,
};
use pin_project::pin_project;
use tower::{BoxError, Layer, Service};

/// Layer that applies [`BaseUri`] which makes all requests relative to the URI.
///
/// Path in the base URI is preseved.
#[derive(Debug, Clone)]
pub struct CacheLayer<K>
where
    K: Resource + 'static,
    K::DynamicType: Eq + PartialEq + Hash + fmt::Debug,
{
    store: Arc<Store<K>>,
}

impl<K> CacheLayer<K>
where
    K: Resource + 'static + Sync + Send,
    K::DynamicType: Eq + PartialEq + Hash + fmt::Debug + Sync + Send,
{
    /// Set base URI of requests.
    pub fn new(store: Store<K>) -> Self {
        Self {
            store: Arc::new(store),
        }
    }
}

impl<S, K> Layer<S> for CacheLayer<K>
where
    K: Resource + 'static + Sync + Send,
    K::DynamicType: Eq + PartialEq + Hash + fmt::Debug + Sync + Send,
{
    type Service = Cache<S, K>;

    fn layer(&self, inner: S) -> Self::Service {
        Cache {
            inner,
            store: self.store.clone(),
        }
    }
}

/// Middleware that sets base URI so that all requests are relative to it.
#[derive(Debug, Clone)]
pub struct Cache<S, K>
where
    K: Resource + 'static + Sync + Send,
    K::DynamicType: Eq + PartialEq + Hash + fmt::Debug + Sync + Send,
{
    inner: S,
    store: Arc<Store<K>>,
}

// impl<S, ReqBody, K> Service<Request<ReqBody>> for Cache<S, K>
// where
//     S: Service<Request<ReqBody>> + Send,
//     K: Sync + Send,
//     // B: http_body::Body<Data = bytes::Bytes> + Send + 'static,
//     // B::Error: Into<BoxError>,
//     S::Response: http_body::Body<Data = bytes::Bytes> + Send + 'static,
//     K: Resource + Clone + 'static + fmt::Debug,
//     K::DynamicType: Eq + PartialEq + Hash + fmt::Debug + Clone + Default,
// {
//     type Error = S::Error;
//     // type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>> ;
//     type Future = S::Future;
//     type Response = S::Response;

//     fn poll_ready(
//         &mut self,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         self.inner.poll_ready(cx)
//     }

//     fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
//         let (parts, body) = req.into_parts();
//         if parts.method != Method::GET {
//             return self.inner.call(Request::from_parts(parts, body));
//         }

//         let path: String = parts.uri.path().into();
//         let req = Request::from_parts(parts, body);
//         let path: Vec<_> = path.splitn(8, "/").collect();
//         let r = match path.deref() {
//             &[_, "api" | "apis", group, version, "namespaces", ns, plural, name]
//                 if group == K::group(&Default::default())
//                     && version == K::version(&Default::default())
//                     && plural == K::plural(&Default::default()) =>
//             {
//                 // let kind = plural.strip_suffix("s");
//                 // let gvk = GroupVersionKind::gvk(group, version, kind.unwrap_or(plural));
//                 ObjectRef::new(name).within(ns)
//                 // ObjectRef::new_with(name, ApiResource::from_gvk(&gvk)).within(ns)
//                 // return CacheResponse{
//                 //     response_future: self.inner,
//                 //     object_ref: ObjectRef::new(name).within(ns),
//                 //     store: self.store,
//                 // }
//             }
//             &[_, "api" | "apis", group, version, plural, name]
//                 if group == K::group(&Default::default())
//                     && version == K::version(&Default::default())
//                     && plural == K::plural(&Default::default()) =>
//             {
//                 // let kind = plural.strip_suffix("s");
//                 // let gvk = GroupVersionKind::gvk(group, version, kind.unwrap_or(plural));
//                 ObjectRef::new(name)
//                 // ObjectRef::new_with(name, ApiResource::from_gvk(&gvk))
//             }
//             _ => {
//                 return self.inner.call(req);
//             }
//         };
//         dbg!(self.store.len(), self.store.get(&dbg!(r)));
//         // Box::pin(async move { Ok(self.store.get(&r).unwrap()) });
//         // parts.uri = dbg!(set_base_uri(&parts.uri, req_pandq));
//         self.inner.call(req)

//         // to_future(Ok(Response::new(bytes::Bytes::new())))
//     }
// }

async fn to_future<R, E>(data: Result<R, E>) -> Result<R, E> {
    data
}
// struct CacheResponse<K, F>
// where
//     K: Resource + 'static + Clone,
//     K::DynamicType: Eq + PartialEq + Hash + Clone,
// {
//     obj: K,
// }

// impl<F, K, Response, Error> Future for CacheResponse<K, F>
// where
//     F: Future<Output = Result<Response, Error>>,
//     Error: Into<BoxError>,
//     Response: http_body::Body<Data = bytes::Bytes> + Send + 'static,
//     K: Resource + 'static + Clone,
//     K::DynamicType: Eq + PartialEq + Hash + Clone,
// {
//     type Output = Result<Response, BoxError>;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         Poll::Ready(Ok(self.obj))
//     }
// }

impl<S, K> Service<Request<Body>> for Cache<S, K>
where
    S: Service<Request<Body>, Response = Response<Incoming>, Error = hyper_util::client::legacy::Error> + Send,
    K: Sync + Send,
    K: Resource + Clone + fmt::Debug,
    K::DynamicType: Eq + PartialEq + Hash + fmt::Debug + Clone + Default,
    S::Future: Sync + Send,
    // where
    //     S: Service<Request<ReqBody>> + Send,
    //     K: Sync + Send,
    //     // B: http_body::Body<Data = bytes::Bytes> + Send + 'static,
    //     // B::Error: Into<BoxError>,
    //     S::Response: http_body::Body<Data = bytes::Bytes> + Send + 'static,
    //     K: Resource + Clone + 'static + fmt::Debug,
    //     K::DynamicType: Eq + PartialEq + Hash + fmt::Debug + Clone + Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Incoming>, hyper_util::client::legacy::Error>> + Send>>;
    // type Future = BoxFuture<'static, Result<Response<Incoming>, hyper_util::client::legacy::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (parts, body) = req.into_parts();
        if parts.method != Method::GET {
            return Box::pin(
                async move { self.inner.call(Request::from_parts(parts, body)).await },
            );
        }

        let path: String = parts.uri.path().into();
        let req = Request::from_parts(parts, body);
        let path: Vec<_> = path.splitn(8, "/").collect();
        let r = match path.deref() {
            &[_, "api" | "apis", group, version, "namespaces", ns, plural, name]
                if group == K::group(&Default::default())
                    && version == K::version(&Default::default())
                    && plural == K::plural(&Default::default()) =>
            {
                // let kind = plural.strip_suffix("s");
                // let gvk = GroupVersionKind::gvk(group, version, kind.unwrap_or(plural));
                ObjectRef::new(name).within(ns)
                // ObjectRef::new_with(name, ApiResource::from_gvk(&gvk)).within(ns)
                // return CacheResponse{
                //     response_future: self.inner,
                //     object_ref: ObjectRef::new(name).within(ns),
                //     store: self.store,
                // }
            }
            &[_, "api" | "apis", group, version, plural, name]
                if group == K::group(&Default::default())
                    && version == K::version(&Default::default())
                    && plural == K::plural(&Default::default()) =>
            {
                // let kind = plural.strip_suffix("s");
                // let gvk = GroupVersionKind::gvk(group, version, kind.unwrap_or(plural));
                ObjectRef::new(name)
                // ObjectRef::new_with(name, ApiResource::from_gvk(&gvk))
            }
            _ => {
                let pin = Box::pin(async move { self.inner.call(req).await });
                return pin;
            }
        };


        dbg!(self.store.len(), self.store.get(&dbg!(r)));
        // return Box::pin(async move { Ok(self.store.get(&r).unwrap()) });
        // parts.uri = dbg!(set_base_uri(&parts.uri, req_pandq));
        return Box::pin(self.inner.call(req));

        // to_future(Ok(Response::new(bytes::Bytes::new())))
    }
}

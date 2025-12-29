use tonic_mock::{streaming_request_lazy, stream_to_vec, test_utils};
use tokio::runtime::Runtime;
use tonic::{Response};

#[test]
fn lazy_stream_produces_expected_messages() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let messages = test_utils::create_test_messages_reuse_data(5, 10);
        let req = streaming_request_lazy(messages.clone());

        let stream = req.into_inner();
        // convert the streaming body into the expected Response<StreamResponseInner<T>>
        let response = Response::new(Box::pin(stream) as tonic_mock::StreamResponseInner<_>);
        let v = stream_to_vec(response).await;
        assert_eq!(v.len(), 5);
    });
}

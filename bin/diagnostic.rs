use std::time::Instant;

mod common;
use common::{TestMessage, TestResponse, process_streaming_response, stream_to_vec, streaming_request, test_utils};
use prost::Message;
use bytes::BufMut;
use tokio::runtime::Runtime;
use tonic_mock::streaming_request_lazy;

fn main() {
    let rt = Runtime::new().unwrap();

    // Parameters to profile
    let counts = [1000usize];
    let sizes = [1000usize];

    for &count in &counts {
        for &size in &sizes {
            println!("\n=== Diagnostic: count={} size={} ===", count, size);

            // Measure message creation
            let t0 = Instant::now();
            let mut messages = Vec::with_capacity(count);
            for i in 0..count {
                let data = "x".repeat(size);
                messages.push(TestMessage::new(i.to_string(), data));
            }
            let msg_create = t0.elapsed();
            println!("message creation: {:?}", msg_create);

            // Measure streaming_request (consumes messages)
            let mut messages_clone = messages.clone();
            let t1 = Instant::now();
            let req = streaming_request(messages_clone.drain(..).collect());
            let req_build = t1.elapsed();
            println!("streaming_request build (encode path): {:?}", req_build);

            // Measure lazy streaming_request that defers encoding
            let t1l = Instant::now();
            let req_lazy = streaming_request_lazy(messages.clone());
            let req_lazy_build = t1l.elapsed();
            println!("streaming_request_lazy build (deferred encode): {:?}", req_lazy_build);

            // Measure prost encoding cost directly (encoded_len + encode into buffer)
            let t_enc = Instant::now();
            for m in &messages {
                let encoded_len = m.encoded_len();
                let mut buf = bytes::BytesMut::with_capacity(encoded_len + 5);
                buf.put_u8(0);
                buf.put_u32(encoded_len as u32);
                m.encode(&mut buf).unwrap();
                let _ = buf.freeze();
            }
            let prost_encode = t_enc.elapsed();
            println!("prost encode (all messages): {:?}", prost_encode);

            // Measure create_stream_response for TestResponse
            let t2 = Instant::now();
            let responses: Vec<TestResponse> = (0..count).map(|i| TestResponse::new(i as i32, "x".repeat(size))).collect();
            let stream_response = test_utils::create_stream_response(responses);
            let stream_build = t2.elapsed();
            println!("create_stream_response: {:?}", stream_build);

            // Measure process_streaming_response with no-op callback
            let t3 = Instant::now();
            rt.block_on(async {
                process_streaming_response(stream_response, |_, _| {}).await;
            });
            let proc_time = t3.elapsed();
            println!("process_streaming_response (no-op): {:?}", proc_time);

            // Measure stream_to_vec end-to-end
            let responses2: Vec<TestResponse> = (0..count).map(|i| TestResponse::new(i as i32, "x".repeat(size))).collect();
            let stream_response2 = test_utils::create_stream_response(responses2);
            let t4 = Instant::now();
            rt.block_on(async {
                let _v = stream_to_vec(stream_response2).await;
            });
            let vec_time = t4.elapsed();
            println!("stream_to_vec: {:?}", vec_time);
        }
    }
}

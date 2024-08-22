# Firehose Ethereum Rust Client

## firehose-ethereum Execution Layer gRPC

### proto files

- [`streamingfast/proto/sf/firehose/v2/firehose.proto`](https://github.com/streamingfast/proto/blob/develop/sf/firehose/v2/firehose.proto)
- [`streamingfast/firehose-ethereum/proto/sf/ethereum/type/v2/type.proto`](https://github.com/streamingfast/firehose-ethereum/blob/335607aac766f9f3c6946d8b1ad3c8e36ab70930/proto/sf/ethereum/type/v2/type.proto)

### gRPC service examples

```terminal
grpcurl -plaintext <your-grpc-service>:<port> describe

grpc.health.v1.Health is a service:
service Health {
  rpc Check ( .grpc.health.v1.HealthCheckRequest ) returns ( .grpc.health.v1.HealthCheckResponse );
  rpc Watch ( .grpc.health.v1.HealthCheckRequest ) returns ( stream .grpc.health.v1.HealthCheckResponse );
}
grpc.reflection.v1.ServerReflection is a service:
service ServerReflection {
  rpc ServerReflectionInfo ( stream .grpc.reflection.v1.ServerReflectionRequest ) returns ( stream .grpc.reflection.v1.ServerReflectionResponse );
}
grpc.reflection.v1alpha.ServerReflection is a service:
service ServerReflection {
  rpc ServerReflectionInfo ( stream .grpc.reflection.v1alpha.ServerReflectionRequest ) returns ( stream .grpc.reflection.v1alpha.ServerReflectionResponse );
}
sf.firehose.v1.Stream is a service:
service Stream {
  rpc Blocks ( .sf.firehose.v1.Request ) returns ( stream .sf.firehose.v1.Response );
}
sf.firehose.v2.Fetch is a service:
service Fetch {
  rpc Block ( .sf.firehose.v2.SingleBlockRequest ) returns ( .sf.firehose.v2.SingleBlockResponse );
}
sf.firehose.v2.Stream is a service:
service Stream {
  rpc Blocks ( .sf.firehose.v2.Request ) returns ( stream .sf.firehose.v2.Response );
}
```

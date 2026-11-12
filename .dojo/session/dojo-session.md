# Dojo Session
mode: autonomous
rigor: real
type: feature
wave: 3
step: RED
gate_density: standard
wave_ceiling: 4
intent: Give the owner a self-hosted manga server/reader (Koma) by executing the mandatory Slice 0 decision pass, starting with a licensed, positioned repo.
goal: Axum server with bearer-token middleware (users table, one seeded user) + tonic/tonic-web unary gRPC-Web route; CORS restricted to deployed UI origins. Integration test: RPC without token → 401; with token → unary response over gRPC-Web.
commit_style: conventional
test_written:
test_status:
attempts: 0
pre_existing_failures: none

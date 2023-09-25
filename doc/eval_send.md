# Evaluation for sending

## Communication additional payload for ID

164 B

Steps:

1. Run `eval-gen-sender-pk-sk` and follow the output to set envs
2. Run `eval-eems-for-send`
3. Run `eval-msg-id-size` and get the result from the output

## Time of ID generation

Message body size: $2^3$ - $2^{13}$ (B)

Local gRPC is set up

Steps:

1. Run `eval-gen-sender-pk-sk` and follow the output to set envs
2. Run `eval-eems-for-send`
3. Run `cargo bench --bench gen_id` and get the result from the output
4. Run `cargo bench --bench gen_id_by_entities` and get the result from the output

- `gen_id` raw log: [gen_id.log](log/gen_id.log)
- `gen_id_by_entities` raw log: [gen_id_by_entities.log](log/gen_id_by_entities.log)

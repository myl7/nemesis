# Evaluation for sending

## Communication additional payload for ID

164 B

Steps:

1. Run `eval-gen-pk-sk` and follow the output to set envs for the sender and EEMS
2. Run `eval-eems-for-send`
3. Run `eval-msg-id-size` and get the result from the output

## Time of ID generation

Message body size: $2^3$ - $2^{13}$ (B)

Local gRPC is set up for `gen_id`.
Time for every entity is evaluated separately for `gen_id_by_entities`.

Steps:

1. Run `eval-gen-pk-sk` and follow the output to set envs for the sender and EEMS
2. Run `eval-eems-for-send`
3. Run `cargo bench --bench gen_id` and get the result from the output
4. Run `cargo bench --bench gen_id_by_entities` and get the result from the output

- `gen_id` raw log: [gen_id.log](log/gen_id.log)
- `gen_id_by_entities` raw log: [gen_id_by_entities.log](log/gen_id_by_entities.log)

## Time of message verification

Message body size: $2^3$ - $2^{13}$ and 1300 - (=)4000 (step by 300) (B)

Steps:

1. Run `eval-gen-pk-sk` and follow the output to set envs for the sender and EEMS
2. Run `eval-eems-for-send`
3. Run `cargo bench --bench verify_msg` and get the result from the output

- `verify_msg` raw log: [verify_msg.log](log/verify_msg.log)

## Time of MSP ID shuffle

Message number (Vector size): `1, 2, 5, 10, 12 * 3600s * 11/s`

Steps:

1. Run `cargo bench --bench shuffle_id` and get the result from the output

- `shuffle_id` raw log: [shuffle_id.log](log/shuffle_id.log)
- `shuffle_id` raw log: [shuffle_id_a1.log](log/shuffle_id_a1.log)

## Communication payload for ID shares

For 2 shares in total: 11312 B

Steps:

1. Run `eval-id-share-size` and get the result from the output

## Time of ID share generation

Steps:

1. Run `cargo bench --bench gen_id_shares` and get the result from the output

- `gen_id_shares` raw log: [gen_id_shares.log](log/gen_id_shares.log)

<!--
## Time of ID shuffle generation

Including doing the shuffle to get the shuffled ID vector

Steps:

1. Run `cargo bench --bench gen_id_shuffle` and get the result from the output

- `gen_id_shuffle` raw log: [gen_id_shuffle.log](log/gen_id_shuffle.log)
 -->

## Time of MSP update

Steps:

1. Run `cargo bench --bench update_msp` and get the result from the output

- `update_msp` raw log: [update_msp.log](log/update_msp.log)

## Time of MSP comparison

Steps:

1. Run `cargo bench --bench msp_compare` and get the result from the output

- `msp_compare` raw log: [msp_compare.log](log/msp_compare.log)

# SinkContract

The sink-carbon contract is the primary interface to use Stellarcarbon on Soroban.
To do an atomic swap of CARBON for CarbonSINK, call the `sink_carbon` function.
The contract also includes several supporting and admin functions.

## Mercury Catch-up

Deploying a new retroshade contract will drop any existing event tables.
To fill a new table with recent events (within the RPC retention window), run:

```sh
mercury-cli \
  --key $MERCURY_KEY \
  --mainnet false \
  --local false \
  catchup \
  --retroshades true \
  --project-name "sorocarbon" \
  --contracts "$(stellar contract alias show sink --network=testnet)" \
  --functions "sink_carbon" \
  --start=<FIRST_TX_LEDGER>
```

You may need to customize this command if you've customized the deployment of your retroshade.

## Interactive Testing

We can do some interactive testing with a contract that is deployed on testnet.
The instructions assume that you've created a stellar-cli alias named `sinker`.
Generate this keypair with `stellar keys generate sinker`.

### fn is_active

```sh
stellar contract invoke \
  --network testnet \
  --source sinker \
  --id sink \
  -- \
  is_active
```

Expected output is `true`:

```text
ℹ️  Send skipped because simulation identified as read-only. Send by rerunning with `--send=yes`.
true
```

### fn get_minimum_amount

```sh
stellar contract invoke \
  --network testnet \
  --source sinker \
  --id sink \
  -- \
  get_minimum_sink_amount
```

Expected output is an integer with value of 1_000_000 or less (0.1 CARBON):

```text
ℹ️  Send skipped because simulation identified as read-only. Send by rerunning with `--send=yes`.
1000000
```

### fn sink_carbon

There are several ways to trigger contract errors.
The first does not require the `funder` and `recipient` account(s) to be created on the network.

Try to sink an amount lower than the minimum:

```sh
stellar contract invoke \
  --network testnet \
  --source sinker \
  --id sink \
  -- \
  sink_carbon \
  --funder "$(stellar keys address sinker)" \
  --recipient "$(stellar keys address sinker)" \
  --amount "0" \
  --project_id "VCS1360" \
  --memo_text "no-op ledger bloat" \
  --email "account@domain.xyz"
```

This should fail with `AmountTooLow`:

```text
❌ error: transaction simulation failed: HostError: Error(Contract, #1067)

Event log (newest first):
   0: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[error, Error(Contract, #1067)], data:"escalating Ok(ScErrorType::Contract) frame-exit to Err"
   1: [Diagnostic Event] topics:[fn_call, CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, sink_carbon], data:[GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, 0, VCS1360, "no-op ledger bloat", "account@domain.xyz"]
```

The following examples can all be executed with the same invoke parameters:

```sh
stellar contract invoke \
  --network testnet \
  --source sinker \
  --id sink \
  -- \
  sink_carbon \
  --funder "$(stellar keys address sinker)" \
  --recipient "$(stellar keys address sinker)" \
  --amount "1000000" \
  --project_id "VCS1360" \
  --memo_text "first" \
  --email "account@domain.xyz"
```

Try to sink without CARBON trustline on funder, expected `AccountOrTrustlineMissing`:

```text
❌ error: transaction simulation failed: HostError: Error(Contract, #1070)

Event log (newest first):
   0: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[error, Error(Contract, #1070)], data:"escalating Ok(ScErrorType::Contract) frame-exit to Err"
   1: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[error, Error(Contract, #13)], data:["contract try_call failed", burn, [GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, 1000000]]
   2: [Failed Diagnostic Event (not emitted)] contract:CCVMSAUB5RSCN7VFA2GESPVGRBNDHLQG5YDA7DST63OXJB5YBZGKEUVU, topics:[error, Error(Contract, #13)], data:["trustline entry is missing for account", GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL]
   3: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[fn_call, CCVMSAUB5RSCN7VFA2GESPVGRBNDHLQG5YDA7DST63OXJB5YBZGKEUVU, burn], data:[GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, 1000000]
   4: [Diagnostic Event] topics:[fn_call, CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, sink_carbon], data:[GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, 1000000, VCS1360, "first", "account@domain.xyz"]
```

Try to sink with insufficient CARBON balance on funder, expected `InsufficientBalance`:

```text
❌ error: transaction simulation failed: HostError: Error(Contract, #1069)

Event log (newest first):
   0: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[error, Error(Contract, #1069)], data:"escalating Ok(ScErrorType::Contract) frame-exit to Err"
   1: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[error, Error(Contract, #10)], data:["contract try_call failed", burn, [GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, 1000000]]
   2: [Failed Diagnostic Event (not emitted)] contract:CCVMSAUB5RSCN7VFA2GESPVGRBNDHLQG5YDA7DST63OXJB5YBZGKEUVU, topics:[error, Error(Contract, #10)], data:["resulting balance is not within the allowed range", 0, -1000000, 9223372036854775807]
   3: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[fn_call, CCVMSAUB5RSCN7VFA2GESPVGRBNDHLQG5YDA7DST63OXJB5YBZGKEUVU, burn], data:[GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, 1000000]
   4: [Diagnostic Event] topics:[fn_call, CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, sink_carbon], data:[GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, 1000000, VCS1360, "first", "account@domain.xyz"]
```

Try to sink without CarbonSINK trustline on the recipient, expected `AccountOrTrustlineMissing`:

```text
❌ error: transaction simulation failed: HostError: Error(Contract, #1070)

Event log (newest first):
   0: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[error, Error(Contract, #1070)], data:"escalating Ok(ScErrorType::Contract) frame-exit to Err"
   1: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[error, Error(Contract, #13)], data:["contract try_call failed", set_authorized, [GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, true]]
   2: [Failed Diagnostic Event (not emitted)] contract:CCUQDX22YTF72Q2F5C4HZSWVMBFTPTLIYXOC3BSNTBSZVJWKMMNUOWXH, topics:[error, Error(Contract, #13)], data:["trustline entry is missing for account", GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL]
   3: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[fn_call, CCUQDX22YTF72Q2F5C4HZSWVMBFTPTLIYXOC3BSNTBSZVJWKMMNUOWXH, set_authorized], data:[GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, true]
   4: [Diagnostic Event] contract:CCVMSAUB5RSCN7VFA2GESPVGRBNDHLQG5YDA7DST63OXJB5YBZGKEUVU, topics:[fn_return, burn], data:Void
   5: [Contract Event] contract:CCVMSAUB5RSCN7VFA2GESPVGRBNDHLQG5YDA7DST63OXJB5YBZGKEUVU, topics:[burn, GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, "CARBON:GDT5XM5C5STQZS5R3F4CEGKJWKDVWBIWBEV4TIYV5MDVVMKA775T4OKY"], data:1000000
   6: [Diagnostic Event] contract:CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, topics:[fn_call, CCVMSAUB5RSCN7VFA2GESPVGRBNDHLQG5YDA7DST63OXJB5YBZGKEUVU, burn], data:[GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, 1000000]
   7: [Diagnostic Event] topics:[fn_call, CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV, sink_carbon], data:[GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL, 1000000, VCS1360, "first", "account@domain.xyz"]
```

And finally, call `sink_carbon` again with all preconditions in place:

```text
ℹ️  Signing transaction: 592a39e5070fcc2fa31d02cf971cc3f14992e3add8e3731afae4d572c9dcc0e6
📅 CCVMSAUB5RSCN7VFA2GESPVGRBNDHLQG5YDA7DST63OXJB5YBZGKEUVU - Event: [{"symbol":"burn"},{"address":"GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL"},{"string":"CARBON:GDT5XM5C5STQZS5R3F4CEGKJWKDVWBIWBEV4TIYV5MDVVMKA775T4OKY"}] = {"i128":{"hi":0,"lo":1000000}}
📅 CCUQDX22YTF72Q2F5C4HZSWVMBFTPTLIYXOC3BSNTBSZVJWKMMNUOWXH - Event: [{"symbol":"set_authorized"},{"address":"CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV"},{"address":"GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL"},{"string":"CarbonSINK:GBO66IRGFZE7UP7MAM5H5IBMZLTM64XE6YNOL4KSL2BFVH7JW6AEKZHO"}] = {"bool":true}
📅 CCUQDX22YTF72Q2F5C4HZSWVMBFTPTLIYXOC3BSNTBSZVJWKMMNUOWXH - Event: [{"symbol":"mint"},{"address":"CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV"},{"address":"GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL"},{"string":"CarbonSINK:GBO66IRGFZE7UP7MAM5H5IBMZLTM64XE6YNOL4KSL2BFVH7JW6AEKZHO"}] = {"i128":{"hi":0,"lo":1000000}}
📅 CCUQDX22YTF72Q2F5C4HZSWVMBFTPTLIYXOC3BSNTBSZVJWKMMNUOWXH - Event: [{"symbol":"set_authorized"},{"address":"CCAVKAYFAUAG7NROYUQGN5DGYKLAMPG4J4D7ZMBUQHLND5K6JIMEZBZV"},{"address":"GAN4SL6DHOQO4POKWOUL4PPCIVJBSDX7SVOLL4GVM4CC27S6WCV7FQZL"},{"string":"CarbonSINK:GBO66IRGFZE7UP7MAM5H5IBMZLTM64XE6YNOL4KSL2BFVH7JW6AEKZHO"}] = {"bool":false}
```

The events that are emitted by the two SACs reveal what happens inside the `sink_carbon` function.
First, the `funder` burns the `amount` of CARBON.
Then, the `recipient`'s CarbonSINK trustline/balance is authorized,
and the same `amount` of CarbonSINK is minted for the `recipient`.
Finally, the CarbonSINK trustline is deauthorized again to lock the balance.

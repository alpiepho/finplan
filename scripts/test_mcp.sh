#!/bin/bash
# Live integration test for the MCP server over stdio.
# Sends JSON-RPC messages to the running server and pretty-prints responses.
set -e
cd "$(dirname "$0")/.."

S=0.3  # delay between messages

(
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
sleep $S
echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
sleep $S

# ── Plan 1: scenario building ──────────────────────────────────────────────────
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"set_parameters","arguments":{"birth_date":"1975-06-15","state_abbreviation":"CA","returns_mode":"historical"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"set_portfolio","arguments":{"name":"My Retirement Plan"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"add_account","arguments":{"name":"Checking","account_type":"Checking","value":25000}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"add_account","arguments":{"name":"401k","account_type":"Traditional401k","assets":[{"ticker":"FXAIX","value":150000}]}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"map_tickers","arguments":{}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"add_income_event","arguments":{"name":"Salary","to_account":"Checking","amount":6000,"interval":"monthly","end_event":"Retirement"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"add_expense_event","arguments":{"name":"Living Expenses","from_account":"Checking","amount":5000,"interval":"monthly"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"add_retirement_event","arguments":{"age":65}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"add_sweep_event","arguments":{"name":"Post-Retirement Spending","to_account":"Checking","target_balance":60000,"start_event":"Retirement"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"add_social_security_event","arguments":{"monthly_amount":2500,"start_age":67,"to_account":"Checking"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"add_rmd_event","arguments":{"destination":"Checking"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"validate_scenario","arguments":{}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"merge_scenario","arguments":{}}}'
sleep $S

# ── Plan 2: simulation & results ───────────────────────────────────────────────
echo '{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"run_simulation","arguments":{}}}'
sleep 2
echo '{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"run_monte_carlo","arguments":{"iterations":50}}}'
sleep 5
echo '{"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":"get_account_snapshot","arguments":{"year":2035}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{"name":"get_account_snapshot","arguments":{"year":2035,"real":true}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":19,"method":"tools/call","params":{"name":"get_ledger","arguments":{"filter":"income","limit":5}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"get_ledger","arguments":{"filter":"expense","year":2030,"limit":5}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{"name":"get_state_summary","arguments":{}}}'
sleep $S
) | cargo run --bin finplan-mcp 2>/dev/null | python3 -c "
import sys, json

def fmt_text(txt, max_len=400):
    txt = txt.strip()
    if len(txt) > max_len:
        txt = txt[:max_len] + '...'
    return txt

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        d = json.loads(line)
    except json.JSONDecodeError:
        print(f'  (non-JSON: {line[:80]})')
        continue

    rid = d.get('id', '?')
    if 'result' in d:
        r = d['result']
        if 'content' in r:
            for item in r['content']:
                txt = item.get('text', '')
                is_err = r.get('is_error', False)
                prefix = 'ERR' if is_err else 'OK '
                print(f'  [{rid}] {prefix}  {fmt_text(txt)}')
        elif 'serverInfo' in r:
            print(f'  [{rid}] INIT  server={r[\"serverInfo\"][\"name\"]}')
        else:
            print(f'  [{rid}] OK    {list(r.keys())}')
    elif 'error' in d:
        print(f'  [{rid}] ERR   {d[\"error\"]}')
"

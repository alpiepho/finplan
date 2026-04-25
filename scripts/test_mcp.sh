#!/bin/bash
# Integration test for the MCP server
set -e

S=0.3  # delay between messages

(
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
sleep $S
echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
sleep $S
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"set_parameters","arguments":{"birth_date":"1985-06-15","state_abbreviation":"CA","returns_mode":"historical"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"set_portfolio","arguments":{"name":"My Retirement Plan"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"add_account","arguments":{"name":"Checking","account_type":"Checking","value":25000}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"add_account","arguments":{"name":"401k","account_type":"Traditional401k","assets":[{"ticker":"FXAIX","value":50000}]}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"map_tickers","arguments":{}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"add_income_event","arguments":{"name":"Salary","to_account":"Checking","amount":4000,"interval":"biweekly","end_event":"Retirement"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"add_expense_event","arguments":{"name":"Living Expenses","from_account":"Checking","amount":5000,"interval":"monthly"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"add_retirement_event","arguments":{"age":65}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"add_sweep_event","arguments":{"name":"Post-Retirement Spending","to_account":"Checking","target_balance":60000,"start_event":"Retirement"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"add_social_security_event","arguments":{"monthly_amount":2500,"start_age":67}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"add_rmd_event","arguments":{}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"validate_scenario","arguments":{}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"merge_scenario","arguments":{}}}'
sleep 0.5
) | docker compose run --rm -T finplan-mcp 2>/dev/null | python3 -c "
import sys, json
for l in sys.stdin:
    l = l.strip()
    if not l: continue
    d = json.loads(l)
    rid = d.get('id', '?')
    if 'result' in d:
        r = d['result']
        if 'content' in r:
            txt = r['content'][0].get('text','')[:300]
            print(f'id={rid}: {txt}')
        elif 'serverInfo' in r:
            print(f'id={rid}: INIT OK')
        else:
            keys = list(r.keys())
            print(f'id={rid}: {keys}')
    elif 'error' in d:
        print(f'id={rid}: ERROR - {d[\"error\"]}')
"

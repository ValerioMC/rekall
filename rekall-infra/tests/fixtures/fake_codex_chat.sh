#!/bin/sh
[ "$1" = app-server ] || exit 4

send_event() {
    printf '{"method":"%s","params":%s}\n' "$1" "$2"
}

finish_turn() {
    send_event 'turn/completed' '{"threadId":"thread-1","turn":{"id":"turn-'"$turn"'", "status":"'"$1"'","durationMs":21}}'
}

answer_turn() {
    send_event 'item/agentMessage/delta' '{"threadId":"thread-1","itemId":"message-'"$turn"'","delta":"'"$1"'"}'
    send_event 'item/completed' '{"threadId":"thread-1","item":{"type":"agentMessage","id":"message-'"$turn"'","text":"'"$1"'"}}'
    send_event 'thread/tokenUsage/updated' '{"threadId":"thread-1","turnId":"turn-'"$turn"'","tokenUsage":{"total":{"totalTokens":'"$((turn * 50))"',"inputTokens":'"$((turn * 40))"',"cachedInputTokens":'"$((turn * 5))"',"outputTokens":'"$((turn * 10))"'},"last":{"totalTokens":50,"inputTokens":40,"cachedInputTokens":5,"outputTokens":10},"modelContextWindow":1000}}'
    finish_turn completed
}

request_approval() {
    send_event 'item/started' '{"threadId":"thread-1","item":{"type":"commandExecution","id":"shell","command":"pwd","status":"inProgress"}}'
    printf '%s\n' '{"id":71,"method":"item/commandExecution/requestApproval","params":{"threadId":"thread-1","turnId":"turn-'"$turn"'","itemId":"shell","command":"pwd"}}'
    printf '%s\n' '{"id":"file-72","method":"item/fileChange/requestApproval","params":{"threadId":"thread-1","turnId":"turn-'"$turn"'","itemId":"file","reason":"Write a file","grantRoot":"."}}'
}

change_files() {
    changes='[{"path":"/work/notes.md","kind":{"type":"add"},"diff":"hello\n"},{"path":"/work/old.md","kind":{"type":"update","move_path":"/work/new.md"},"diff":"@@ -1 +1 @@\n-a\n+b\n"}]'
    send_event 'item/started' '{"threadId":"thread-1","item":{"type":"fileChange","id":"patch","status":"inProgress","changes":'"$changes"'}}'
    send_event 'item/completed' '{"threadId":"thread-1","item":{"type":"fileChange","id":"patch","status":"completed","changes":'"$changes"'}}'
    answer_turn 'Files changed.'
}

request_mcp_approval() {
    printf '%s\n' '{"id":73,"method":"mcpServer/elicitation/request","params":{"threadId":"thread-1","turnId":"turn-'"$turn"'","serverName":"rekall","mode":"form","_meta":{"codex_approval_kind":"mcp_tool_call","tool_name":"rekall_context","tool_params":{"anchors":"task:AL-1"}},"requestedSchema":{"type":"object","properties":{}}}}'
    printf '%s\n' '{"id":74,"method":"mcpServer/elicitation/request","params":{"threadId":"thread-1","turnId":"turn-'"$turn"'","serverName":"rekall","mode":"form","_meta":{"codex_approval_kind":"mcp_tool_call","tool_name":"rekall_step","tool_params":{"state":"claimed"}},"requestedSchema":{"type":"object","properties":{}}}}'
}

turn=0
while IFS= read -r line; do
    if [ -n "$FAKE_CODEX_RECEIVED" ]; then
        printf '%s\n' "$line" >> "$FAKE_CODEX_RECEIVED"
    fi
    case "$line" in
        *'"method":"initialize"'*)
            printf '%s\n' '{"id":1,"result":{"userAgent":"codex-test/0.172"}}'
            ;;
        *'"method":"thread/start"'*)
            if [ "$FAKE_CODEX_CHAT_UNSUPPORTED" = 1 ]; then
                printf '%s\n' '{"id":2,"error":{"code":-32601,"message":"Unknown method thread/start"}}'
            else
                printf '{"id":2,"result":{"thread":{"id":"thread-1"},"model":"gpt-test","cwd":"%s","approvalPolicy":"on-request"}}\n' "$PWD"
            fi
            ;;
        *'"method":"turn/start"'*)
            id=$(printf '%s\n' "$line" | sed -E 's/.*"id":([0-9]+).*/\1/')
            turn=$((turn + 1))
            printf '{"id":%s,"result":{"turn":{"id":"turn-%s","status":"inProgress"}}}\n' "$id" "$turn"
            send_event 'turn/started' '{"threadId":"thread-1","turn":{"id":"turn-'"$turn"'","status":"inProgress"}}'
            case "$line" in
                *'refine'*) answer_turn 'Which region should host the service?' ;;
                *'request approval'*) request_approval ;;
                *'request MCP approval'*) request_mcp_approval ;;
                *'change files'*) change_files ;;
                *'hold this turn'*) ;;
                *'eu-west-1'*) answer_turn 'I will use eu-west-1.' ;;
                *) answer_turn 'Ready.' ;;
            esac
            ;;
        *'"method":"turn/interrupt"'*)
            id=$(printf '%s\n' "$line" | sed -E 's/.*"id":([0-9]+).*/\1/')
            printf '{"id":%s,"result":{}}\n' "$id"
            finish_turn interrupted
            ;;
        *'"id":71'*'"decision":"accept"'*)
            send_event 'item/completed' '{"threadId":"thread-1","item":{"type":"commandExecution","id":"shell","command":"pwd","status":"completed","exitCode":0,"aggregatedOutput":"/work"}}'
            send_event 'serverRequest/resolved' '{"threadId":"thread-1","requestId":71}'
            ;;
        *'"id":"file-72"'*'"decision":"decline"'*)
            answer_turn 'I will leave the file alone.'
            ;;
        *'"id":74'*'"action":"decline"'*)
            answer_turn 'The step was left open.'
            ;;
    esac
done

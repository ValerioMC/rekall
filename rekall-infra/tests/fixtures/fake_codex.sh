#!/bin/sh
case "$1:$2:$3" in
  mcp:add:rekall)
    [ "$4" = "--url" ] || exit 3
    printf '%s' "$5" > .mcp-url
    ;;
  mcp:list:--json)
    if [ -f .mcp-url ]; then
      printf '[{"name":"rekall","enabled":true,"transport":{"url":"%s"}}]\n' "$(cat .mcp-url)"
    else
      printf '[]\n'
    fi
    ;;
  app-server::)
    while IFS= read -r line; do
      case "$line" in
        *'"method":"initialize"'*)
          printf '%s\n' '{"id":1,"result":{}}'
          ;;
        *'"method":"model/list"'*'"cursor":"next"'*)
          printf '%s\n' '{"id":3,"result":{"data":[{"model":"test-2","displayName":"Test 2","supportedReasoningEfforts":[]}],"nextCursor":null}}'
          ;;
        *'"method":"model/list"'*)
          printf '%s\n' '{"method":"trace","params":{}}'
          printf '%s\n' '{"id":2,"result":{"data":[{"model":"test-1","displayName":"Test 1","supportedReasoningEfforts":[{"reasoningEffort":"high"}]}],"nextCursor":"next"}}'
          ;;
        *'"method":"account/rateLimits/read"'*)
          printf '%s\n' '{"id":2,"result":{"rateLimits":{"primary":{"usedPercent":27,"resetsAt":null,"windowDurationMins":300},"secondary":{"usedPercent":52.5,"resetsAt":1999999999,"windowDurationMins":10080}}}}'
          ;;
      esac
    done
    ;;
  *) exit 4 ;;
esac

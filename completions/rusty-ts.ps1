
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'rusty-ts' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'rusty-ts'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'rusty-ts' {
            [CompletionResult]::new('--tz', '--tz', [CompletionResultType]::ParameterName, 'Render timestamps in the named IANA timezone (e.g., `America/New_York`). Resolved once at startup; per-line render cost is a fixed-offset conversion. Rejected in Strict mode')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'Render elapsed time since the previous input line instead of absolute time')
            [CompletionResult]::new('--incremental', '--incremental', [CompletionResultType]::ParameterName, 'Render elapsed time since the previous input line instead of absolute time')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Render elapsed time since program start instead of absolute time')
            [CompletionResult]::new('--since-start', '--since-start', [CompletionResultType]::ParameterName, 'Render elapsed time since program start instead of absolute time')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'Use a monotonic clock source for elapsed-time calculations. Has no effect unless `-i` or `-s` is also present')
            [CompletionResult]::new('--monotonic', '--monotonic', [CompletionResultType]::ParameterName, 'Use a monotonic clock source for elapsed-time calculations. Has no effect unless `-i` or `-s` is also present')
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'Convert recognized in-line timestamps to relative form rather than prefixing new timestamps. Default mode recognizes ISO-8601, RFC-3339, and Unix epoch; Strict mode expands to the full moreutils set')
            [CompletionResult]::new('--relative', '--relative', [CompletionResultType]::ParameterName, 'Convert recognized in-line timestamps to relative form rather than prefixing new timestamps. Default mode recognizes ISO-8601, RFC-3339, and Unix epoch; Strict mode expands to the full moreutils set')
            [CompletionResult]::new('-u', '-u', [CompletionResultType]::ParameterName, 'Force timestamps to be rendered in UTC, overriding system local time and the `TZ` env var. Rejected in Strict mode')
            [CompletionResult]::new('--utc', '--utc', [CompletionResultType]::ParameterName, 'Force timestamps to be rendered in UTC, overriding system local time and the `TZ` env var. Rejected in Strict mode')
            [CompletionResult]::new('--strict', '--strict', [CompletionResultType]::ParameterName, 'Switch into Strict moreutils Compatibility Mode. Rejects `-u`, `--tz`, and other Rusty-only flags; expands `-r` to the full moreutils set; mirrors moreutils `--help` / `--version` layout; ignores `RUSTY_TS_FORMAT`')
            [CompletionResult]::new('--no-strict', '--no-strict', [CompletionResultType]::ParameterName, 'Force Default mode, overriding `RUSTY_TS_STRICT` env var and argv[0] auto-detection')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell-completion scripts for bash, zsh, fish, or powershell. Writes to stdout')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rusty-ts;completions' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'rusty-ts;help' {
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell-completion scripts for bash, zsh, fish, or powershell. Writes to stdout')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rusty-ts;help;completions' {
            break
        }
        'rusty-ts;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}

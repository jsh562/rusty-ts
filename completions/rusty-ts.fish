# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_rusty_ts_global_optspecs
	string join \n i/incremental s/since-start m/monotonic r/relative u/utc tz= strict no-strict h/help V/version
end

function __fish_rusty_ts_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_rusty_ts_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_rusty_ts_using_subcommand
	set -l cmd (__fish_rusty_ts_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -l tz -d 'Render timestamps in the named IANA timezone (e.g., `America/New_York`). Resolved once at startup; per-line render cost is a fixed-offset conversion. Rejected in Strict mode' -r
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -s i -l incremental -d 'Render elapsed time since the previous input line instead of absolute time'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -s s -l since-start -d 'Render elapsed time since program start instead of absolute time'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -s m -l monotonic -d 'Use a monotonic clock source for elapsed-time calculations. Has no effect unless `-i` or `-s` is also present'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -s r -l relative -d 'Convert recognized in-line timestamps to relative form rather than prefixing new timestamps. Default mode recognizes ISO-8601, RFC-3339, and Unix epoch; Strict mode expands to the full moreutils set'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -s u -l utc -d 'Force timestamps to be rendered in UTC, overriding system local time and the `TZ` env var. Rejected in Strict mode'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -l strict -d 'Switch into Strict moreutils Compatibility Mode. Rejects `-u`, `--tz`, and other Rusty-only flags; expands `-r` to the full moreutils set; mirrors moreutils `--help` / `--version` layout; ignores `RUSTY_TS_FORMAT`'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -l no-strict -d 'Force Default mode, overriding `RUSTY_TS_STRICT` env var and argv[0] auto-detection'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -s h -l help -d 'Print help'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -s V -l version -d 'Print version'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -a "completions" -d 'Generate shell-completion scripts for bash, zsh, fish, or powershell. Writes to stdout'
complete -c rusty-ts -n "__fish_rusty_ts_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c rusty-ts -n "__fish_rusty_ts_using_subcommand completions" -s h -l help -d 'Print help'
complete -c rusty-ts -n "__fish_rusty_ts_using_subcommand help; and not __fish_seen_subcommand_from completions help" -f -a "completions" -d 'Generate shell-completion scripts for bash, zsh, fish, or powershell. Writes to stdout'
complete -c rusty-ts -n "__fish_rusty_ts_using_subcommand help; and not __fish_seen_subcommand_from completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'

; Injection queries for the standarx DSL.
;
; No language injections by default — multi-line templates often
; hold shell scripts (`cmd "bun run build"`) but we don't have a
; reliable signal to detect that here. Downstream editor configs
; can layer their own injection rules per project (e.g. injecting
; bash into a template that lives under a `cmd` key).
;
; Reserved as a placeholder so editor extensions can find the file
; and override it.

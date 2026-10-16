# Cheap markdownlint subset (no node needed): MD012 consecutive blanks, MD022 blanks around headings,
# MD032 blanks before lists, MD033 bare <tag> outside backticks. Usage: awk -f tools/md_lint.awk file.md ...
# Prints "file:line RULE message"; exit 1 when anything is found. Code fences are skipped.
function report(rule, msg) { printf "%s:%d %s %s\n", FILENAME, FNR, rule, msg; found = 1 }
function is_list(s) { return s ~ /^[ \t]*([-*+]|[0-9]+\.)[ \t]/ }
FNR == 1 { prev = ""; prev_blank = 1; prev_heading = 0; fence = 0; front = 0 }
FNR == 1 && /^---[ \t]*$/ { front = 1; next }
front { if ($0 ~ /^---[ \t]*$/) { front = 0; prev_blank = 0 } ; next }
/^[ \t]*(```|~~~)/ { fence = !fence; prev = $0; prev_blank = 0; prev_heading = 0; next }
fence { next }
{
	blank = ($0 ~ /^[ \t]*$/)
	heading = ($0 ~ /^#{1,6} /)
	if (blank && prev_blank && FNR > 1) report("MD012", "multiple consecutive blank lines")
	if (heading && !prev_blank) report("MD022", "heading needs a blank line above")
	if (!blank && prev_heading) report("MD022", "heading needs a blank line below")
	if (is_list($0) && !prev_blank && !is_list(prev) && prev !~ /^[ \t]+/ && prev !~ /^\|/) report("MD032", "list needs a blank line above")
	line = $0
	gsub(/`[^`]*`/, "", line)
	if (line ~ /<[A-Za-z][A-Za-z0-9_-]*>/ && line !~ /<(br|kbd|sub|sup|details|summary)>/) report("MD033", "bare <tag>: wrap it in backticks")
	prev = $0; prev_blank = blank; prev_heading = heading
}
END { exit found }

#!/usr/bin/env bash
# Print every colour slot the Alacritty port sets, using ANSI escapes only, so
# what you see is what the terminal actually loaded rather than what this script
# sends. Run it after switching flavours.
set -euo pipefail

r=$'\e[0m'
sgr() { printf '\e[%sm' "$1"; }

names=(black red green yellow blue magenta cyan white)

printf '\n  %s\n\n' "Acid — ANSI slots"

printf '  %-10s' 'normal'
for i in {0..7}; do printf '%s %-3d %s' "$(sgr "48;5;$i")" "$i" "$r"; done
printf '\n  %-10s' 'bright'
for i in {8..15}; do printf '%s %-3d %s' "$(sgr "48;5;$i")" "$i" "$r"; done
printf '\n  %-10s' 'dim'
for i in {0..7}; do printf '%s %s %s' "$(sgr "2;48;5;$i")" '   ' "$r"; done
printf '\n  %-10s%s 16  %s  orange, no ANSI slot of its own\n\n' 'indexed' "$(sgr '48;5;16')" "$r"

printf '  %-10s' 'on bg'
for i in {0..7}; do printf '%s%-8s%s' "$(sgr "38;5;$i")" "${names[$i]}" "$r"; done
printf '\n  %-10s' 'bright'
for i in {8..15}; do printf '%s%-8s%s' "$(sgr "38;5;$i")" "${names[$((i - 8))]}" "$r"; done
printf '\n\n'

# A sample that puts the accents where a syntax highlighter would.
kw=$(sgr '38;5;5')    # purple: keywords
fn=$(sgr '38;5;6')    # aqua:   functions
str=$(sgr '38;5;2')   # green:  strings
num=$(sgr '38;5;16')  # orange: numbers
ty=$(sgr '38;5;3')    # yellow: types
var=$(sgr '38;5;4')   # blue:   identifiers
cm=$(sgr '38;5;8')    # comment
err=$(sgr '38;5;1')   # red:    errors

cat <<SAMPLE
  ${cm}// Every accent doing the job a highlighter would give it.${r}
  ${kw}pub const fn${r} ${fn}new${r}(${var}value${r}: ${ty}u32${r}) -> ${ty}Self${r} {
      ${kw}let${r} ${var}mask${r} = ${num}0x00ff_ffff${r};
      ${kw}if${r} ${var}value${r} > ${var}mask${r} { ${kw}return${r} ${fn}Err${r}(${str}"out of range"${r}); }
      ${fn}Self${r}(${var}value${r} & ${var}mask${r})
  }
  ${err}error${r}: ${cm}this line is what a diagnostic looks like${r}

SAMPLE
printf '  %sreverse video%s  %sbold%s  %sitalic%s  %sunderline%s  %sstrikethrough%s\n\n' \
  "$(sgr 7)" "$r" "$(sgr 1)" "$r" "$(sgr 3)" "$r" "$(sgr 4)" "$r" "$(sgr 9)" "$r"

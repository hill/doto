# doto todo

doto is a really simple plain text todo list manager written in Rust.

<blockquote class="twitter-tweet"><p lang="en" dir="ltr">✅ My productivity app is a file called TODO.TXT<br><br>Features:<br>- 🪆 infinite nesting by using [tab]<br>- ♻️ versioning via git commits<br>- 📲 multi-platform via git<br>- 💰 costs $0/month + forever free <a href="https://t.co/fCsA2wC2a8">pic.twitter.com/fCsA2wC2a8</a></p>&mdash; @levelsio (@levelsio) <a href="https://twitter.com/levelsio/status/1545387078816497672?ref_src=twsrc%5Etfw">July 8, 2022</a></blockquote> <script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>

All todo files are plaintext `.md` files that look like this:

```
# 2023-05-24
- [x] buy belinda a gift (2023-05-23)
- [ ] train tickets on sunday
- [ ] respond to lily

## pgMagic
- [ ] fix SQL parsing bug
- [>] get licence checks working (2023-05-28)
```

and are saved in your `$DOTO_PATH` directory or `~/.doto`. You can sync this to git or do whatever your want with it.

Open today's todo with

`$ doto`

Move all previously undone tasks to today

`$ doto --undone`

Open tomorrow

`$ doto tomorrow` (`$doto tom`)

Open any day in the previous week:

`$ doto tue`

Open any day this month:

`$ doto 21` (to open the 21st this month)

Open a specific date:

`$ doto 2010-12-24`

# doto todo ✅

doto is a really simple plain text todo list manager written in Rust.

<center>
<img src="./tweet.png" height="500" />
</center>

---

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

You can style the text todo files however you would like as long as the task begins with `- [ ]` (to enable task counting and moving capabilities)

The files are saved in your `$DOTO_PATH` directory or `~/.doto`. You can sync this to git or do whatever your want with it.

Open today's todo with

`$ doto`

Move all previously undone tasks to today

`$ doto --undone`

Open tomorrow

`$ doto tomorrow` (`$ doto tom`)

Open any day in the previous week:

`$ doto tue`

Open any day this month:

`$ doto 21` (to open the 21st this month)

Open a specific date:

`$ doto 2010-12-24`

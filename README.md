## lila-http

Take some of the HTTP load away from lila. WIP!

### Arena tournaments

Clients connected to a [tournament page](https://lichess.org/tournament/winter21)
request new data about the tournament every 4s or so, with XHR HTTP requests.

Each player requests information about a different leaderboard page: the one they're in.

When a tournament has 17k connected clients, like it happened during the
[Agadmator Arena](https://lichess.org/@/Lichess/blog/our-recent-server-issues/FdKHVehW),
then [lila](https://github.com/ornicar/lila) has to serve about 5k tournament update requests
per second.

It's too much. Even tho most of the data is cached by lila, these requests are authenticated
and have a cost. lila usually serves at most 2k requests per second, and is not designed to
suddenly serve 5k/s more.

So, the plan is to have a new service handle these tournament update requests.
It gets info about ongoing tournaments from lila, and propagates it to the clients.

### lila-ws, and now lila-http

Much like [lila-ws](https://github.com/ornicar/lila-ws) moved the websocket traffic away from lila,
lila-http handles some of the heavy HTTP traffic.

It may be expanded to other areas than just the arena tournaments in the future.

### Optional service

One goal of lila-http is to be optional. Lichess should work just fine without it.
It means that lila and lila-http can handle the same requests in the same way.

This simplifies dev environments, which won't need to install lila-http,
and makes production more resilient to lila-http restarts or outages.

This goal is a nice-to-have, not a must-have, and might be dropped in the future
if it proves to be too inconvenient.

### Why Rust

It could have been done in scala, like lila-ws. But I saw this as an opportunity to learn rust,
which I know is a fantastic language.

### Why not [language]

I value strong static typing very highly, and both scala and rust have it. Haskell would be an other option.
Go, not so much.

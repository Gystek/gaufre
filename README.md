gaufre - a gopher client
========================
gaufre is a UN*X-only line-oriented terminal gopher client
theoretically compatible with every terminal supporting basic ANSI
colour codes.  It is not that fast, it may be a bit buggy, but it
mostly works fine.


Requirements
------------
In order to build gaufre you need a Rust environment and a "make"
program.


Installation
------------
Edit `config.mk` to match your local setup (gaufre is installed into the
`/usr/local` namespace by default).

Edit `config.h` (copy `config.def.h` if you don't know what to do) to
customize the software to suit your needs. Yes, I'm too lazy to make a
decent configuration system.

Afterwards enter the following command to build and install gaufre (if
necessary as root):

    make install


Running gaufre
--------------
Use it with the following syntax

    gaufre HOST[:PORT}

Type `help` during your session if you are lost.


TODO
----

- Refactor the `link` function (amongst other things)
- Add caching support (maybe)


Credits
-------
Might have burrowed (pun intended) the menu display from
kieselsteini/cgo.

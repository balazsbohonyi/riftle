Riftle Portable
================

Run riftle.exe from this folder.

Portable mode is enabled by the riftle-launcher.portable marker file next to
riftle.exe. Keep both files together.

In portable mode, Riftle stores settings, the search database, and extracted
icons in a local data folder beside riftle.exe:

  .\data\

Do not delete riftle-launcher.portable unless you want Riftle to use the
installed-mode data directory instead.

Launch at startup is not available in the current portable UI. If startup
support is enabled for portable builds in the future, disable it before moving
or deleting the portable folder so Windows does not keep a stale startup entry.

Riftle uses Microsoft Edge WebView2. Most current Windows systems already have
WebView2 installed. If this portable build does not start, install or update
the Microsoft Edge WebView2 Runtime and try again.

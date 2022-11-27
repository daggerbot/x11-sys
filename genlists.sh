#!/bin/sh
set -e

if [ ! "$1" = "--yesiamsure" ]; then
    printf "WARNING: This script only works on Debian with all of the relevant packages installed.\n" >&2
    printf "WARNING: It will also overwrite everything in the lists/ directory.\n" >&2
    printf "WARNING: Run with \`--yesiamsure\` to execute this script anyway.\n" >&2
    exit 1
fi

libdir=/usr/lib/x86_64-linux-gnu
deb_pkgs="glx x11 x11-xcb xcursor xext xfixes xft xi xinerama xmu xpresent xrandr xrender xss xt xtst xxf86vm"

for pkg in $deb_pkgs; do
    header_list="lists/${pkg}-headers.txt"
    function_list="lists/${pkg}-functions.txt"
    var_list="lists/${pkg}-vars.txt"

    # Clear previous header list because we generate it by appending.
    rm -f $header_list

    # Manual headers for the beginning of the header list.
    # This is necessary because some headers don't properly include their dependencies.
    case $pkg in
        x11) printf "X11/Xlib.h\n" >$header_list ;;
        xext) printf "X11/Xlib.h\nX11/Xproto.h\n" >$header_list ;;
        xt) printf "X11/IntrinsicP.h\n" >$header_list ;;
        xxf86vm) printf "X11/Xlib.h\n" >$header_list ;;
    esac

    # Get debian package name containing headers.
    case $pkg in
        xmu) debpkg=libxmu-headers ;;
        *) debpkg="lib${pkg}-dev" ;;
    esac

    # Generate header list from debian package.
    for header in $(dpkg -L $debpkg | egrep '/include/.*\.h$' | sed 's@.*/include/@@' | LC_ALL=C sort); do
        case $header in
            # Note: Some headers are included by other headers but should not be included directly from our code.
            # Don't add them to the header list.
            X11/CallbackI.h | X11/InitialI.h | X11/TranslateI.h) ;;
            *) printf "$header\n" >>$header_list
        esac
    done

    # Get pkg-config package name.
    # These are usually the same as the debian/feature package names, but not always.
    case $pkg in
        xss) pcpkg=xscrnsaver ;;
        *) pcpkg=$pkg ;;
    esac

    # Get library name for this package.
    libname="$(pkg-config --libs $pcpkg | sed -e 's/^-l//' -e 's/ .*//')"
    lib="${libdir}/lib${libname}.so"

    # Read symbol names from library.
    readelf -sW "$lib" | grep ' FUNC ' | egrep ' [0-9]+ [A-Za-z0-9_]+$' | sed 's/.* //' | LC_ALL=C sort >$function_list
    readelf -sW "$lib" | grep ' OBJECT ' | egrep ' [0-9]+ [A-Za-z0-9_]+$' | sed 's/.* //' | LC_ALL=C sort >$var_list
done

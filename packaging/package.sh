#!/bin/bash

function clean() {
    echo Cleaning up...
    rm -f $PACKAGE_DIR/tmp -r
    rm -f $PACKAGE_DIR/crabfetch
    rm -f $PACKAGE_DIR/LICENCE
}


PACKAGE_DIR=$(dirname `realpath $0`)
ROOT_DIR=$(dirname $PACKAGE_DIR)
echo "Using code directory: $ROOT_DIR"
echo "Using packaging directory: $PACKAGE_DIR"

read -p "Do you want to proceed? (yes/no) " yn
case $yn in 
	yes ) 
        ;;
    no ) 
        clean
        exit 1
        ;;
    * ) 
        clean
        exit 1
        ;;
esac


echo "Building CrabFetch..."
cd $ROOT_DIR
cargo build -r -F music,rpm_package
cp $ROOT_DIR/target/release/crabfetch $PACKAGE_DIR
cp $ROOT_DIR/LICENCE $PACKAGE_DIR
cd $PACKAGE_DIR
cp $PACKAGE_DIR/crabfetch $PACKAGE_DIR/crabfetch-unstripped
strip $PACKAGE_DIR/crabfetch

# holy moly bash this is terrible
IFS=' '
read -ra VERSION_ARR <<< $($PACKAGE_DIR/crabfetch --version)
VERSION_STRING=${VERSION_ARR[1]}

EXECUTABLE_CHECKSUM=$(sha256sum $PACKAGE_DIR/crabfetch)
EXECUTABLE_CHECKSUM=${EXECUTABLE_CHECKSUM%% *}
LICENCE_CHECKSUM=$(sha256sum $PACKAGE_DIR/LICENCE)
LICENCE_CHECKSUM=${LICENCE_CHECKSUM%% *}
echo "Version String: $VERSION_STRING"
echo "Binary Checksum: $EXECUTABLE_CHECKSUM"
echo "Licence Checksum: $LICENCE_CHECKSUM"

read -p "Do you want to proceed? (yes/no) " yn
case $yn in 
	yes ) ;;
	no ) exit;;
	* ) exit;;
esac


#
# Generic File Structure
#
mkdir $PACKAGE_DIR/tmp/struct/usr/bin -p
mkdir $PACKAGE_DIR/tmp/struct/usr/share/licenses/crabfetch -p
cp $PACKAGE_DIR/crabfetch $PACKAGE_DIR/tmp/struct/usr/bin
cp $PACKAGE_DIR/LICENCE $PACKAGE_DIR/tmp/struct/usr/share/licenses/crabfetch/LICENCE



#
# TARBALL
#
mkdir $PACKAGE_DIR/tmp/tarball
cp $PACKAGE_DIR/crabfetch $PACKAGE_DIR/tmp/tarball
cp $PACKAGE_DIR/LICENCE $PACKAGE_DIR/tmp/tarball
cd $PACKAGE_DIR/tmp/tarball
tar -cf "crabfetch-${VERSION_STRING}_amd64.tar.gz" *
mv $PACKAGE_DIR/tmp/tarball/crabfetch-${VERSION_STRING}_amd64.tar.gz $PACKAGE_DIR
cd $PACKAGE_DIR


#
# DEBIAN
#
echo Building Debian...
mkdir $PACKAGE_DIR/tmp/deb/DEBIAN/ -p
touch $PACKAGE_DIR/tmp/deb/DEBIAN/control
echo "Package: crab-fetch" >> $PACKAGE_DIR/tmp/deb/DEBIAN/control
echo "Description: Extremely fast, featureful and customizable command-line fetcher." >> $PACKAGE_DIR/tmp/deb/DEBIAN/control
echo "Version: $VERSION_STRING" >> $PACKAGE_DIR/tmp/deb/DEBIAN/control
echo "Maintainer: Livaco <livaco@livaco.dev>" >> $PACKAGE_DIR/tmp/deb/DEBIAN/control
echo "Architecture: amd64" >> $PACKAGE_DIR/tmp/deb/DEBIAN/control
echo "Section: utils" >> $PACKAGE_DIR/tmp/deb/DEBIAN/control
echo "Depends: glibc" >> $PACKAGE_DIR/tmp/deb/DEBIAN/control
echo "Recommends: glxinfo, pciutils" >> $PACKAGE_DIR/tmp/deb/DEBIAN/control

touch $PACKAGE_DIR/tmp/deb/DEBIAN/sha256sums
DEBIAN_EXEC_SUM="$EXECUTABLE_CHECKSUM /usr/bin/crabfetch"
DEBIAN_LICENCE_SUM="$LICENCE_CHECKSUM /usr/share/licenses/crabfetch/LICENCE"
echo $DEBIAN_EXEC_SUM >> $PACKAGE_DIR/tmp/deb/DEBIAN/sha256sums
echo $DEBIAN_LICENCE_SUM >> $PACKAGE_DIR/tmp/deb/DEBIAN/sha256sums

cp $PACKAGE_DIR/tmp/struct/* $PACKAGE_DIR/tmp/deb/ -r 

dpkg-deb --build $PACKAGE_DIR/tmp/deb
mv "$PACKAGE_DIR/tmp/deb.deb" "$PACKAGE_DIR/crabfetch-${VERSION_STRING}_amd64.deb"


# Raw binary
cp "$PACKAGE_DIR/crabfetch" "$PACKAGE_DIR/crabfetch-${VERSION_STRING}_amd64"

clean

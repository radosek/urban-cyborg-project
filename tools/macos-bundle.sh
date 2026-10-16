#!/bin/sh
# Builds a universal (arm64 + x86_64) URBAN.app and, when credentials are available, signs it
# with a Developer ID and notarizes it.
#   tools/macos-bundle.sh <out-dir>
# Signing:      ad-hoc by default (free; runs on Apple Silicon, macOS asks once on first open).
#               SIGN_IDENTITY="Developer ID Application: …" signs with a Developer ID instead.
# Notarizing:   NOTARY_KEY (path to .p8), NOTARY_KEY_ID, NOTARY_ISSUER   (App Store Connect API key)
#          or   NOTARY_PROFILE (a `xcrun notarytool store-credentials` profile)
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=${1:-$ROOT/dist/macos}
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/rust/Cargo.toml" | head -1)
mkdir -p "$OUT"
cd "$ROOT/rust"
for t in aarch64-apple-darwin x86_64-apple-darwin; do
	MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release --target $t
done
APP="$OUT/URBAN.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/urban" \
	target/aarch64-apple-darwin/release/urban target/x86_64-apple-darwin/release/urban
sed "s/@VERSION@/$VERSION/g" "$ROOT/packaging/macos/Info.plist" > "$APP/Contents/Info.plist"
cp "$ROOT/packaging/macos/urban.icns" "$APP/Contents/Resources/urban.icns"

if [ -n "$SIGN_IDENTITY" ]; then
	codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" "$APP"
else
	codesign --force --sign - "$APP"
fi
codesign --verify --strict --verbose=2 "$APP"

ZIP="$OUT/urban-macos-universal.zip"
rm -f "$ZIP"
if [ -n "$SIGN_IDENTITY" ] && { [ -n "$NOTARY_KEY" ] || [ -n "$NOTARY_PROFILE" ]; }; then
	ditto -c -k --keepParent "$APP" "$OUT/notarize.zip"
	if [ -n "$NOTARY_PROFILE" ]; then
		xcrun notarytool submit "$OUT/notarize.zip" --keychain-profile "$NOTARY_PROFILE" --wait
	else
		xcrun notarytool submit "$OUT/notarize.zip" --key "$NOTARY_KEY" --key-id "$NOTARY_KEY_ID" --issuer "$NOTARY_ISSUER" --wait
	fi
	xcrun stapler staple "$APP"
	spctl --assess --type execute --verbose "$APP"
	rm -f "$OUT/notarize.zip"
fi
cp "$ROOT/README.md" "$ROOT/docs/FIXES.md" "$OUT/"
cp "$ROOT/ref/copying" "$OUT/COPYING"
(cd "$OUT" && ditto -c -k --keepParent URBAN.app urban-macos-universal.zip.tmp && mv urban-macos-universal.zip.tmp urban-macos-universal.zip)
echo "$ZIP"

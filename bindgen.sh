#!/usr/bin/env sh

WHITELIST='(xcb|XCB)_(xim|XIM|im|xic)_.*|xcb_compound_text.*|xcb_utf8_to_compound_text|free'

bindgen \
	--allowlist-function "$WHITELIST" \
	--allowlist-type "_xcb_im_style_t" \
	--allowlist-var "$WHITELIST" \
	--size_t-is-usize \
	--no-layout-tests \
	"xcb-imdkit.h" \
	-o src/bindings.rs \
	-- \
	-Ideps/xcb-imdkit/src \
	-Ideps/xcb-imdkit/uthash \
	-Ideps/xcb-imdkit-generated-headers \
	-std=c99 \
	-D_GNU_SOURCE \
	-Dxcb_imdkit_EXPORTS

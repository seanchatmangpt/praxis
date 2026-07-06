/**
 * mosaic-native.js
 * -----------------------------------------------------------------------------
 * A dependency-free, in-browser implementation of the React Native primitive
 * surface, with the SAME component names, prop names and flexbox layout
 * semantics as `react-native`. Write your screens against these and the JSX
 * ports to Expo verbatim — at migration time you delete this file and change
 * one import:
 *
 *     // prototype (here)
 *     const { View, Text, Pressable } = window.MosaicNative;
 *     // expo
 *     import { View, Text, Pressable } from 'react-native';
 *
 * Implemented: View, Text, Pressable, TouchableOpacity, TextInput, ScrollView,
 * FlatList, SectionList, Image, ImageBackground, SafeAreaView, ActivityIndicator,
 * Switch, StyleSheet, Platform, Dimensions, useColorScheme, useWindowDimensions,
 * Alert, Linking.
 *
 * Fidelity notes (these match RN, NOT the web defaults):
 *   • every View is flexbox, flexDirection defaults to 'column'
 *   • alignItems defaults to 'stretch'
 *   • flexShrink defaults to 0  (the famous RN "text gets clipped" gotcha)
 *   • numbers are density-independent px; layout props get 'px' appended
 *   • RN-only props expand: padding/marginHorizontal|Vertical, shadow*, gap nums
 * -----------------------------------------------------------------------------
 */
(function (global) {
  'use strict';
  // Wait for the DC runtime's React global, then build the primitive layer
  // against the SAME React instance the rest of the page uses.
  if (!global.React) {
    var _tries = 0;
    var _wait = setInterval(function () {
      if (global.React) { clearInterval(_wait); build(global.React); }
      else if (++_tries > 200) { clearInterval(_wait); console.error('mosaic-native: React global never appeared'); }
    }, 25);
    return;
  }
  build(global.React);

  function build(React) {
  var h = React.createElement;

  /* ---- one-time injected CSS (placeholder color + spinner + scrollbar) ---- */
  (function injectCSS() {
    if (document.getElementById('mosaic-native-css')) return;
    var s = document.createElement('style');
    s.id = 'mosaic-native-css';
    s.textContent =
      '.mn-input::placeholder{color:var(--mn-ph,rgba(0,0,0,0.3));}' +
      '.mn-input{outline:none;border:none;background:transparent;font:inherit;color:inherit;margin:0;padding:0;width:100%;box-sizing:border-box;}' +
      '.mn-noscroll::-webkit-scrollbar{display:none;}' +
      '@keyframes mn-spin{to{transform:rotate(360deg);}}';
    document.head.appendChild(s);
  })();

  /* ---- style engine: RN style object(s) → CSS style object ---- */
  // props whose numeric values are unitless in CSS
  var UNITLESS = {
    flex: 1, flexGrow: 1, flexShrink: 1, opacity: 1, zIndex: 1, order: 1,
    fontWeight: 1, aspectRatio: 1, shadowOpacity: 1, elevation: 1, scale: 1,
  };
  // RN composite props handled specially (never copied verbatim)
  var SPECIAL = {
    paddingHorizontal: 1, paddingVertical: 1, marginHorizontal: 1, marginVertical: 1,
    shadowColor: 1, shadowOffset: 1, shadowOpacity: 1, shadowRadius: 1, elevation: 1,
    flex: 1, tintColor: 1, transform: 1, textAlignVertical: 1,
  };

  function flattenStyle(style) {
    if (!style) return null;
    if (Array.isArray(style)) {
      var out = {};
      for (var i = 0; i < style.length; i++) {
        var f = flattenStyle(style[i]);
        if (f) for (var k in f) out[k] = f[k];
      }
      return out;
    }
    return style;
  }

  function len(v) { return typeof v === 'number' ? v + 'px' : v; }

  // Convert an RN style object into a CSS-in-JS object honoring RN semantics.
  function toCss(style, opts) {
    var rn = flattenStyle(style) || {};
    var css = {};
    opts = opts || {};

    // RN flexbox defaults (only for box-type elements)
    if (opts.box) {
      css.display = 'flex';
      css.flexDirection = 'column';
      css.alignContent = 'flex-start';
      css.flexShrink = 0;
      css.flexBasis = 'auto';
      css.boxSizing = 'border-box';
      css.position = 'relative';
      css.minHeight = 0;
      css.minWidth = 0;
    }

    for (var key in rn) {
      var val = rn[key];
      if (val == null) continue;
      if (SPECIAL[key]) continue; // handled below
      if (typeof val === 'number' && !UNITLESS[key]) css[key] = val + 'px';
      else css[key] = val;
    }

    // composite expansions
    if (rn.paddingHorizontal != null) { css.paddingLeft = len(rn.paddingHorizontal); css.paddingRight = len(rn.paddingHorizontal); }
    if (rn.paddingVertical != null) { css.paddingTop = len(rn.paddingVertical); css.paddingBottom = len(rn.paddingVertical); }
    if (rn.marginHorizontal != null) { css.marginLeft = len(rn.marginHorizontal); css.marginRight = len(rn.marginHorizontal); }
    if (rn.marginVertical != null) { css.marginTop = len(rn.marginVertical); css.marginBottom = len(rn.marginVertical); }

    // flex shorthand: RN `flex: n` (n>0) → grow n / shrink 1 / basis 0
    if (rn.flex != null) {
      if (typeof rn.flex === 'number') {
        if (rn.flex > 0) { css.flexGrow = rn.flex; css.flexShrink = 1; css.flexBasis = '0%'; }
        else if (rn.flex === 0) { css.flexGrow = 0; css.flexShrink = 0; css.flexBasis = 'auto'; }
        else { css.flexGrow = 0; css.flexShrink = 1; css.flexBasis = 'auto'; }
      } else css.flex = rn.flex;
    }

    // shadows (iOS shadow* + android elevation) → boxShadow
    if (rn.shadowColor || rn.elevation != null) {
      var off = rn.shadowOffset || { width: 0, height: rn.elevation != null ? rn.elevation / 2 : 1 };
      var blur = rn.shadowRadius != null ? rn.shadowRadius : (rn.elevation != null ? rn.elevation : 4);
      var color = applyOpacity(rn.shadowColor || '#000', rn.shadowOpacity != null ? rn.shadowOpacity : (rn.elevation != null ? 0.18 : 1));
      css.boxShadow = (off.width || 0) + 'px ' + (off.height || 0) + 'px ' + blur + 'px ' + color;
    }

    // transform array → CSS transform string
    if (rn.transform) {
      if (Array.isArray(rn.transform)) {
        css.transform = rn.transform.map(function (t) {
          var p = Object.keys(t)[0]; var v = t[p];
          if (p === 'rotate' || p === 'rotateX' || p === 'rotateY' || p === 'rotateZ') return p + '(' + v + ')';
          if (p === 'translateX' || p === 'translateY') return p + '(' + len(v) + ')';
          if (p === 'scale' || p === 'scaleX' || p === 'scaleY') return p + '(' + v + ')';
          if (p === 'skewX' || p === 'skewY') return p + '(' + v + ')';
          return p + '(' + v + ')';
        }).join(' ');
      } else css.transform = rn.transform;
    }
    return css;
  }

  function applyOpacity(color, op) {
    if (op == null || op === 1) return color;
    if (color[0] === '#') {
      var hex = color.slice(1);
      if (hex.length === 3) hex = hex.split('').map(function (c) { return c + c; }).join('');
      var r = parseInt(hex.slice(0, 2), 16), g = parseInt(hex.slice(2, 4), 16), b = parseInt(hex.slice(4, 6), 16);
      return 'rgba(' + r + ',' + g + ',' + b + ',' + op + ')';
    }
    return color;
  }

  // strip RN-only props before spreading onto a DOM node
  function domProps(props, omit) {
    var out = {};
    for (var k in props) {
      if (k === 'style' || k === 'children') continue;
      if (omit && omit[k]) continue;
      out[k] = props[k];
    }
    return out;
  }

  /* ============================== View =================================== */
  var View = React.forwardRef(function View(props, ref) {
    var omit = { pointerEvents: 1, onLayout: 1, collapsable: 1, needsOffscreenAlphaCompositing: 1 };
    var extra = domProps(props, omit);
    var css = toCss(props.style, { box: true });
    if (props.pointerEvents) css.pointerEvents = props.pointerEvents;
    return h('div', Object.assign({ ref: ref, style: css }, extra), props.children);
  });

  /* ============================== Text =================================== */
  // RN Text is NOT a flex box — it lays out as text and composes when nested.
  var Text = React.forwardRef(function Text(props, ref) {
    var omit = { numberOfLines: 1, onPress: 1, ellipsizeMode: 1, selectable: 1, allowFontScaling: 1 };
    var extra = domProps(props, omit);
    var css = toCss(props.style, { box: false });
    css.margin = css.margin || 0;
    css.boxSizing = 'border-box';
    if (props.numberOfLines === 1) {
      css.whiteSpace = 'nowrap'; css.overflow = 'hidden'; css.textOverflow = 'ellipsis'; css.display = 'block';
    } else if (props.numberOfLines > 1) {
      css.display = '-webkit-box'; css.WebkitLineClamp = props.numberOfLines;
      css.WebkitBoxOrient = 'vertical'; css.overflow = 'hidden';
    }
    if (props.onPress) { extra.onClick = props.onPress; css.cursor = css.cursor || 'pointer'; }
    return h('span', Object.assign({ ref: ref, style: css }, extra), props.children);
  });

  /* ============================ Pressable ================================ */
  function Pressable(props) {
    var st = React.useState(false), pressed = st[0], setPressed = st[1];
    var omit = { onPress: 1, onPressIn: 1, onPressOut: 1, onLongPress: 1, android_ripple: 1, hitSlop: 1, disabled: 1, style: 1, children: 1 };
    var styleProp = typeof props.style === 'function' ? props.style({ pressed: pressed }) : props.style;
    var css = toCss(styleProp, { box: true });
    css.cursor = props.disabled ? 'default' : 'pointer';
    css.userSelect = 'none';
    if (props.disabled) css.opacity = css.opacity != null ? css.opacity : 0.5;
    var children = typeof props.children === 'function' ? props.children({ pressed: pressed }) : props.children;
    var extra = domProps(props, omit);
    return h('div', Object.assign({
      role: 'button', style: css,
      onPointerDown: function () { if (!props.disabled) { setPressed(true); props.onPressIn && props.onPressIn(); } },
      onPointerUp: function () { setPressed(false); props.onPressOut && props.onPressOut(); },
      onPointerLeave: function () { setPressed(false); },
      onClick: function (e) { if (!props.disabled && props.onPress) props.onPress(e); },
    }, extra), children);
  }

  /* ========================= TouchableOpacity =========================== */
  function TouchableOpacity(props) {
    var st = React.useState(false), down = st[0], setDown = st[1];
    var omit = { onPress: 1, activeOpacity: 1, disabled: 1, style: 1, children: 1, onPressIn: 1, onPressOut: 1 };
    var css = toCss(props.style, { box: true });
    css.cursor = props.disabled ? 'default' : 'pointer';
    css.userSelect = 'none';
    css.transition = 'opacity 0.12s';
    if (down) css.opacity = props.activeOpacity != null ? props.activeOpacity : 0.2;
    else if (props.disabled) css.opacity = 0.5;
    var extra = domProps(props, omit);
    return h('div', Object.assign({
      role: 'button', style: css,
      onPointerDown: function () { if (!props.disabled) setDown(true); },
      onPointerUp: function () { setDown(false); },
      onPointerLeave: function () { setDown(false); },
      onClick: function (e) { if (!props.disabled && props.onPress) props.onPress(e); },
    }, extra), props.children);
  }

  /* ============================ TextInput =============================== */
  var TextInput = React.forwardRef(function TextInput(props, ref) {
    var omit = {
      onChangeText: 1, placeholderTextColor: 1, secureTextEntry: 1, keyboardType: 1,
      multiline: 1, style: 1, value: 1, autoCapitalize: 1, autoCorrect: 1, returnKeyType: 1,
      onSubmitEditing: 1, numberOfLines: 1, blurOnSubmit: 1, selectionColor: 1,
    };
    var css = toCss(props.style, { box: false });
    if (props.placeholderTextColor) css['--mn-ph'] = props.placeholderTextColor;
    var km = { 'number-pad': 'numeric', 'numeric': 'decimal', 'decimal-pad': 'decimal', 'email-address': 'email', 'phone-pad': 'tel', 'url': 'url' };
    var common = Object.assign({
      ref: ref, className: 'mn-input', style: css,
      value: props.value,
      placeholder: props.placeholder,
      inputMode: km[props.keyboardType],
      autoCapitalize: props.autoCapitalize,
      spellCheck: props.autoCorrect === false ? false : undefined,
      onChange: function (e) { props.onChangeText && props.onChangeText(e.target.value); },
      onKeyDown: function (e) { if (e.key === 'Enter' && !props.multiline && props.onSubmitEditing) props.onSubmitEditing({ nativeEvent: { text: e.target.value } }); },
    }, domProps(props, omit));
    if (props.multiline) return h('textarea', Object.assign(common, { rows: props.numberOfLines || 3 }));
    common.type = props.secureTextEntry ? 'password' : 'text';
    return h('input', common);
  });

  /* ============================ ScrollView ============================== */
  var ScrollView = React.forwardRef(function ScrollView(props, ref) {
    var omit = {
      contentContainerStyle: 1, horizontal: 1, showsVerticalScrollIndicator: 1,
      showsHorizontalScrollIndicator: 1, style: 1, children: 1, keyboardShouldPersistTaps: 1,
      onScroll: 1, scrollEventThrottle: 1, refreshControl: 1, stickyHeaderIndices: 1, bounces: 1,
    };
    var outer = toCss(props.style, { box: true });
    outer.overflowX = props.horizontal ? 'auto' : 'hidden';
    outer.overflowY = props.horizontal ? 'hidden' : 'auto';
    outer.WebkitOverflowScrolling = 'touch';
    var hideBar = (props.horizontal ? props.showsHorizontalScrollIndicator : props.showsVerticalScrollIndicator) === false;
    var inner = toCss(props.contentContainerStyle, { box: true });
    if (props.horizontal) inner.flexDirection = inner.flexDirection || 'row';
    var extra = domProps(props, omit);
    if (props.onScroll) extra.onScroll = function (e) { props.onScroll({ nativeEvent: { contentOffset: { x: e.target.scrollLeft, y: e.target.scrollTop } } }); };
    return h('div', Object.assign({ ref: ref, style: outer, className: hideBar ? 'mn-noscroll' : undefined }, extra),
      h('div', { style: inner }, props.children));
  });

  /* ============================== FlatList ============================== */
  function FlatList(props) {
    var data = props.data || [];
    var keyOf = props.keyExtractor || function (item, i) { return (item && (item.id != null ? item.id : item.key)) != null ? (item.id != null ? item.id : item.key) : i; };
    var Sep = props.ItemSeparatorComponent;
    var rows = [];
    for (var i = 0; i < data.length; i++) {
      var item = data[i];
      rows.push(h(React.Fragment, { key: keyOf(item, i) },
        props.renderItem({ item: item, index: i }),
        Sep && i < data.length - 1 ? h(Sep, null) : null
      ));
    }
    var content = props.numColumns && props.numColumns > 1
      ? h('div', { style: { display: 'grid', gridTemplateColumns: 'repeat(' + props.numColumns + ', 1fr)', gap: 0 } }, rows)
      : rows;
    return h(ScrollView, {
      horizontal: props.horizontal,
      style: props.style,
      contentContainerStyle: props.contentContainerStyle,
      showsVerticalScrollIndicator: props.showsVerticalScrollIndicator,
      showsHorizontalScrollIndicator: props.showsHorizontalScrollIndicator,
      onScroll: props.onScroll, scrollEventThrottle: props.scrollEventThrottle,
    },
      props.ListHeaderComponent ? renderElementProp(props.ListHeaderComponent) : null,
      data.length === 0 && props.ListEmptyComponent ? renderElementProp(props.ListEmptyComponent) : content,
      props.ListFooterComponent ? renderElementProp(props.ListFooterComponent) : null
    );
  }

  function SectionList(props) {
    var sections = props.sections || [];
    var blocks = [];
    sections.forEach(function (section, si) {
      if (props.renderSectionHeader) blocks.push(h(React.Fragment, { key: 'h' + si }, props.renderSectionHeader({ section: section })));
      (section.data || []).forEach(function (item, ii) {
        blocks.push(h(React.Fragment, { key: si + '-' + ii }, props.renderItem({ item: item, index: ii, section: section })));
      });
    });
    return h(ScrollView, { style: props.style, contentContainerStyle: props.contentContainerStyle },
      props.ListHeaderComponent ? renderElementProp(props.ListHeaderComponent) : null, blocks);
  }

  function renderElementProp(C) {
    if (!C) return null;
    if (React.isValidElement(C)) return C;
    if (typeof C === 'function') return h(C, null);
    return C;
  }

  /* =============================== Image ================================ */
  function resolveSource(source) {
    if (!source) return null;
    if (typeof source === 'string') return source;
    if (typeof source === 'object' && source.uri) return source.uri;
    return null;
  }
  var FIT = { cover: 'cover', contain: 'contain', stretch: 'fill', center: 'none', repeat: 'cover' };
  var Image = React.forwardRef(function Image(props, ref) {
    var omit = { source: 1, resizeMode: 1, style: 1, tintColor: 1, onLoad: 1, onError: 1, defaultSource: 1 };
    var css = toCss(props.style, { box: false });
    css.objectFit = FIT[props.resizeMode] || 'cover';
    css.display = 'block';
    var uri = resolveSource(props.source);
    if (props.tintColor) {
      // tinted image → mask
      var mcss = Object.assign({}, css, {
        backgroundColor: props.tintColor,
        WebkitMaskImage: 'url(' + uri + ')', maskImage: 'url(' + uri + ')',
        WebkitMaskSize: css.objectFit, maskSize: css.objectFit,
        WebkitMaskRepeat: 'no-repeat', maskRepeat: 'no-repeat',
        WebkitMaskPosition: 'center', maskPosition: 'center',
      });
      return h('div', Object.assign({ ref: ref, style: mss(mcss) }, domProps(props, omit)));
    }
    return h('img', Object.assign({ ref: ref, src: uri, style: css,
      onLoad: props.onLoad, onError: props.onError }, domProps(props, omit)));
  });
  function mss(o) { return o; }

  function ImageBackground(props) {
    var css = toCss(props.style, { box: true });
    var uri = resolveSource(props.source);
    var imgCss = toCss(props.imageStyle, { box: false });
    return h('div', { style: css },
      h('img', { src: uri, style: Object.assign({ position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: FIT[props.resizeMode] || 'cover' }, imgCss) }),
      h('div', { style: { position: 'relative', flex: '1 1 0%', display: 'flex', flexDirection: 'column' } }, props.children)
    );
  }

  /* =========================== SafeAreaView ============================= */
  var SafeAreaView = React.forwardRef(function SafeAreaView(props, ref) {
    var css = toCss(props.style, { box: true });
    if (css.paddingTop == null) css.paddingTop = 'env(safe-area-inset-top, 0px)';
    return h('div', Object.assign({ ref: ref, style: css }, domProps(props, { style: 1, children: 1 })), props.children);
  });

  /* ========================= ActivityIndicator ========================= */
  function ActivityIndicator(props) {
    var size = props.size === 'large' ? 36 : (typeof props.size === 'number' ? props.size : 20);
    var color = props.color || '#999';
    var wrap = toCss(props.style, { box: true });
    wrap.alignItems = wrap.alignItems || 'center';
    wrap.justifyContent = wrap.justifyContent || 'center';
    if (props.animating === false) return h('div', { style: wrap });
    return h('div', { style: wrap },
      h('div', { style: {
        width: size, height: size, borderRadius: size,
        border: Math.max(2, size / 9) + 'px solid ' + applyOpacity(color, 0.25),
        borderTopColor: color, animation: 'mn-spin 0.7s linear infinite',
      } })
    );
  }

  /* ============================== Switch =============================== */
  function Switch(props) {
    var on = !!props.value;
    var trackOn = (props.trackColor && props.trackColor.true) || '#34C759';
    var trackOff = (props.trackColor && props.trackColor.false) || 'rgba(120,120,128,0.32)';
    var thumb = props.thumbColor || '#fff';
    return h('div', {
      role: 'switch', 'aria-checked': on,
      onClick: function () { if (!props.disabled && props.onValueChange) props.onValueChange(!on); },
      style: {
        width: 51, height: 31, borderRadius: 31, padding: 2, cursor: props.disabled ? 'default' : 'pointer',
        background: on ? trackOn : trackOff, transition: 'background 0.2s', boxSizing: 'border-box',
        display: 'flex', flexDirection: 'row', justifyContent: on ? 'flex-end' : 'flex-start',
        opacity: props.disabled ? 0.5 : 1, flexShrink: 0,
      },
    }, h('div', { style: {
      width: 27, height: 27, borderRadius: 27, background: thumb,
      boxShadow: '0 1px 3px rgba(0,0,0,0.3)', transition: 'all 0.2s',
    } }));
  }

  /* ============================= StyleSheet ============================ */
  var StyleSheet = {
    create: function (obj) { return obj; },
    flatten: flattenStyle,
    compose: function (a, b) { return [a, b]; },
    hairlineWidth: 0.5,
    absoluteFillObject: { position: 'absolute', left: 0, right: 0, top: 0, bottom: 0 },
    absoluteFill: { position: 'absolute', left: 0, right: 0, top: 0, bottom: 0 },
  };

  /* ============================== Platform ============================= */
  var Platform = {
    OS: 'web',
    Version: 'web',
    isPad: false, isTV: false,
    select: function (spec) { return spec.web !== undefined ? spec.web : (spec.native !== undefined ? spec.native : spec.default); },
  };

  /* ============================ Dimensions ============================= */
  var _dims = { window: { width: 402, height: 874, scale: 3, fontScale: 1 }, screen: { width: 402, height: 874, scale: 3, fontScale: 1 } };
  var _dimListeners = [];
  var Dimensions = {
    get: function (k) { return _dims[k] || _dims.window; },
    set: function (w, hgt) { _dims.window = { width: w, height: hgt, scale: 3, fontScale: 1 }; _dims.screen = _dims.window; _dimListeners.forEach(function (cb) { cb({ window: _dims.window, screen: _dims.screen }); }); },
    addEventListener: function (type, cb) { _dimListeners.push(cb); return { remove: function () { var i = _dimListeners.indexOf(cb); if (i >= 0) _dimListeners.splice(i, 1); } }; },
  };
  function useWindowDimensions() {
    var st = React.useState(_dims.window), d = st[0], setD = st[1];
    React.useEffect(function () { var sub = Dimensions.addEventListener('change', function (e) { setD(e.window); }); return function () { sub.remove(); }; }, []);
    return d;
  }
  function useColorScheme() { return 'dark'; }

  /* ============================ Alert/Linking ========================== */
  var Alert = {
    alert: function (title, message, buttons) {
      if (buttons && buttons.length > 1) {
        var ok = global.confirm((title ? title + '\n\n' : '') + (message || ''));
        var pick = ok ? buttons.find(function (b) { return b.style !== 'cancel'; }) : buttons.find(function (b) { return b.style === 'cancel'; });
        if (pick && pick.onPress) pick.onPress();
      } else {
        global.alert((title ? title + '\n\n' : '') + (message || ''));
        if (buttons && buttons[0] && buttons[0].onPress) buttons[0].onPress();
      }
    },
  };
  var Linking = {
    openURL: function (url) { global.open(url, '_blank'); return Promise.resolve(); },
    canOpenURL: function () { return Promise.resolve(true); },
  };

  var MosaicNative = {
    View: View, Text: Text, Pressable: Pressable, TouchableOpacity: TouchableOpacity,
    TextInput: TextInput, ScrollView: ScrollView, FlatList: FlatList, SectionList: SectionList,
    Image: Image, ImageBackground: ImageBackground, SafeAreaView: SafeAreaView,
    ActivityIndicator: ActivityIndicator, Switch: Switch,
    StyleSheet: StyleSheet, Platform: Platform, Dimensions: Dimensions,
    useWindowDimensions: useWindowDimensions, useColorScheme: useColorScheme,
    Alert: Alert, Linking: Linking,
  };
  global.MosaicNative = MosaicNative;
  if (typeof module !== 'undefined' && module.exports) module.exports = MosaicNative;
  }
})(typeof window !== 'undefined' ? window : this);

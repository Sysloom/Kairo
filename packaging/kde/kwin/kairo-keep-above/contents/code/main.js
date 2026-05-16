/* global workspace, print */

// KWin owns window stacking on Plasma Wayland. Tauri's alwaysOnTop hint can be
// ignored, so Kairo enforces its overlay timer windows from the compositor side.

(function () {
  var PLUGIN_NAME = "kairo-keep-above";
  var OVERLAY_TITLES = ["Kairo Floating Timer", "Kairo Mini Timer"];
  var watchedWindows = [];

  function asText(value) {
    if (value === undefined || value === null) {
      return "";
    }

    return String(value);
  }

  function titleMatches(window) {
    var caption = asText(window.caption);

    for (var index = 0; index < OVERLAY_TITLES.length; index += 1) {
      if (caption.indexOf(OVERLAY_TITLES[index]) !== -1) {
        return true;
      }
    }

    return false;
  }

  function isKairoOverlay(window) {
    if (!window || window.deleted || window.desktopWindow || window.dock) {
      return false;
    }

    // Title matching is intentional: WebKitGTK/Wayland may not expose a stable
    // resource class before the window is fully mapped, while these captions are
    // unique to Kairo overlay timer windows.
    return titleMatches(window);
  }

  function enforceOverlayPolicy(window) {
    if (!isKairoOverlay(window)) {
      return;
    }

    if (!window.keepAbove) {
      window.keepAbove = true;
    }

    if (!window.skipTaskbar) {
      window.skipTaskbar = true;
    }

    if (!window.skipPager) {
      window.skipPager = true;
    }

    if (!window.skipSwitcher) {
      window.skipSwitcher = true;
    }

    if (hasNonOverlayWindowAbove(window) && workspace && workspace.raiseWindow) {
      workspace.raiseWindow(window);
    }
  }

  function hasNonOverlayWindowAbove(window) {
    var windows = workspace.stackingOrder || [];
    var foundTarget = false;

    for (var index = 0; index < windows.length; index += 1) {
      if (windows[index] === window) {
        foundTarget = true;
        continue;
      }

      if (foundTarget && !isKairoOverlay(windows[index])) {
        return true;
      }
    }

    return false;
  }

  function enforceExistingOverlayWindows() {
    var windows = workspace.stackingOrder || [];

    for (var index = 0; index < windows.length; index += 1) {
      enforceOverlayPolicy(windows[index]);
    }
  }

  function connectIfAvailable(object, signalName, callback) {
    if (object && object[signalName] && object[signalName].connect) {
      object[signalName].connect(callback);
    }
  }

  function watchWindow(window) {
    enforceOverlayPolicy(window);

    if (watchedWindows.indexOf(window) !== -1) {
      return;
    }

    watchedWindows.push(window);

    connectIfAvailable(window, "windowShown", function () {
      enforceOverlayPolicy(window);
      enforceExistingOverlayWindows();
    });

    connectIfAvailable(window, "keepAboveChanged", function () {
      enforceOverlayPolicy(window);
    });

    connectIfAvailable(window, "skipTaskbarChanged", function () {
      enforceOverlayPolicy(window);
    });

    connectIfAvailable(window, "captionChanged", function () {
      enforceOverlayPolicy(window);
    });

    connectIfAvailable(window, "windowClassChanged", function () {
      enforceOverlayPolicy(window);
    });

    connectIfAvailable(window, "activeChanged", function () {
      enforceExistingOverlayWindows();
    });

    connectIfAvailable(window, "stackingOrderChanged", function () {
      enforceExistingOverlayWindows();
    });

    connectIfAvailable(window, "closed", function () {
      var index = watchedWindows.indexOf(window);

      if (index !== -1) {
        watchedWindows.splice(index, 1);
      }
    });
  }

  function watchExistingWindows() {
    var windows = workspace.stackingOrder || [];

    for (var index = 0; index < windows.length; index += 1) {
      watchWindow(windows[index]);
    }
  }

  if (!workspace || !workspace.windowAdded) {
    print(PLUGIN_NAME + ": KWin workspace API unavailable");
    return;
  }

  watchExistingWindows();

  workspace.windowAdded.connect(function (window) {
    watchWindow(window);
    enforceExistingOverlayWindows();
  });

  workspace.windowActivated.connect(function () {
    watchExistingWindows();
    enforceExistingOverlayWindows();
  });

  connectIfAvailable(workspace, "stackingOrderChanged", function () {
    watchExistingWindows();
    enforceExistingOverlayWindows();
  });

  print(PLUGIN_NAME + ": loaded");
})();

use super::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Pointer sample in logical (DIP) coordinates relative to the panel.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelPointerSample {
    pub x: f64,
    pub y: f64,
    pub is_left: bool,
    pub is_right: bool,
}

/// Wheel sample in logical (DIP) coordinates relative to the panel.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelWheelSample {
    pub x: f64,
    pub y: f64,
    pub delta_y: f64,
}

/// Optional pointer / wheel callbacks for
/// [`SwapChainPanelHandle::attach_input`].
#[derive(Clone, Default)]
pub struct SwapChainPanelInput {
    pub on_pointer_down: Option<Callback<PanelPointerSample>>,
    pub on_pointer_move: Option<Callback<PanelPointerSample>>,
    pub on_pointer_up: Option<Callback<PanelPointerSample>>,
    pub on_pointer_enter: Option<Callback<PanelPointerSample>>,
    pub on_pointer_exit: Option<Callback<()>>,
    pub on_wheel: Option<Callback<PanelWheelSample>>,
}

fn panel_pointer_sample(
    args: &bindings::PointerRoutedEventArgs,
    relative_to: &bindings::UIElement,
) -> Option<PanelPointerSample> {
    let point = args.GetCurrentPoint(relative_to).ok()?;
    let pos = point.Position().ok()?;
    let props = point.Properties().ok()?;
    Some(PanelPointerSample {
        x: pos.x as f64,
        y: pos.y as f64,
        is_left: props.IsLeftButtonPressed().unwrap_or(false),
        is_right: props.IsRightButtonPressed().unwrap_or(false),
    })
}

fn panel_wheel_sample(
    args: &bindings::PointerRoutedEventArgs,
    relative_to: &bindings::UIElement,
) -> Option<PanelWheelSample> {
    let point = args.GetCurrentPoint(relative_to).ok()?;
    let pos = point.Position().ok()?;
    let props = point.Properties().ok()?;
    let delta = props.MouseWheelDelta().unwrap_or(0) as f64 / 120.0;
    Some(PanelWheelSample {
        x: pos.x as f64,
        y: pos.y as f64,
        delta_y: delta,
    })
}

/// Opaque handle to the native `SwapChainPanel` control, passed to the
/// [`on_mounted`](SwapChainPanel::on_mounted) callback.
#[derive(Clone)]
pub struct SwapChainPanelHandle(windows_core::IInspectable);

impl sealed::ElementHandle for SwapChainPanelHandle {
    fn from_native(native: windows_core::IInspectable) -> Self {
        Self(native)
    }
}

impl SwapChainPanelHandle {
    /// Wraps a native `SwapChainPanel` created outside [`swap_chain_panel()`].
    pub fn from_native(native: windows_core::IInspectable) -> Self {
        Self(native)
    }

    /// Attach a DXGI swap chain (created with `CreateSwapChainForComposition`).
    ///
    /// # Safety contract
    /// The caller must pass a valid `IDXGISwapChain` (or `IDXGISwapChain1`).
    /// Passing an unrelated COM interface will fail at the WinUI layer.
    pub fn set_swap_chain(&self, swap_chain: &impl Interface) -> Result<()> {
        let native: bindings::ISwapChainPanelNative = self.0.cast()?;
        unsafe { native.SetSwapChain(swap_chain.as_raw()).ok() }
    }

    /// Detaches any swap chain previously attached to the panel.
    pub fn clear_swap_chain(&self) -> Result<()> {
        let native: bindings::ISwapChainPanelNative = self.0.cast()?;
        unsafe { native.SetSwapChain(std::ptr::null_mut()).ok() }
    }

    /// Returns the panel's laid-out size in logical (DIP) units.
    pub fn actual_size(&self) -> Result<(f64, f64)> {
        let fe: bindings::IFrameworkElement = self.0.cast()?;
        Ok((fe.ActualWidth()?, fe.ActualHeight()?))
    }

    /// Returns the host XAML rasterization scale (DPI scale factor).
    ///
    /// Prefer this over [`composition_scale`](Self::composition_scale) for
    /// WinUI 3 desktop hosts: `SwapChainPanel.CompositionScaleX/Y` can remain
    /// `1.0` even when the window is rendered at higher DPI, while
    /// `XamlRoot.RasterizationScale` tracks the effective per-window scale.
    pub fn rasterization_scale(&self) -> Result<f64> {
        let element: bindings::IUIElement = self.0.cast()?;
        let root = element.XamlRoot()?;
        root.RasterizationScale()
    }

    /// Returns the current composition scale as `(scale_x, scale_y)`.
    pub fn composition_scale(&self) -> Result<(f32, f32)> {
        let panel: bindings::ISwapChainPanel = self.0.cast()?;
        let x = panel.CompositionScaleX()?;
        let y = panel.CompositionScaleY()?;
        Ok((x, y))
    }

    /// Subscribes to composition scale changes.
    pub fn on_composition_scale_changed(
        &self,
        f: impl Fn(f32, f32) + 'static,
    ) -> Result<windows_core::EventRevoker> {
        let panel: bindings::ISwapChainPanel = self.0.cast()?;
        panel.CompositionScaleChanged(move |sender, _| {
            if let Some(sender) = sender.as_ref() {
                let scp: &bindings::ISwapChainPanel = sender;
                let x = scp.CompositionScaleX().unwrap_or(1.0);
                let y = scp.CompositionScaleY().unwrap_or(1.0);
                f(x, y);
            }
        })
    }

    /// Subscribes to pointer and wheel events on the native panel.
    ///
    /// Subscriptions live until the panel is destroyed. Call once per mount
    /// (for example from an [`on_mounted`](SwapChainPanel::on_mounted)
    /// callback).
    pub fn attach_input(&self, input: SwapChainPanelInput) -> Result<()> {
        let ui: bindings::UIElement = self.0.cast()?;
        let iue: bindings::IUIElement = ui.cast()?;
        let ui_for_events = ui.clone();
        let revokers: Rc<RefCell<Vec<windows_core::EventRevoker>>> =
            Rc::new(RefCell::new(Vec::new()));

        if let Some(cb) = input.on_pointer_down {
            let ui = ui_for_events.clone();
            revokers.borrow_mut().push(iue.PointerPressed(move |_sender, args| {
                if let Some(args) = args.as_ref()
                    && let Some(sample) = panel_pointer_sample(args, &ui)
                {
                    cb.invoke(sample);
                }
            })?);
        }

        if let Some(cb) = input.on_pointer_up {
            let ui = ui_for_events.clone();
            revokers.borrow_mut().push(iue.PointerReleased(move |_sender, args| {
                if let Some(args) = args.as_ref()
                    && let Some(sample) = panel_pointer_sample(args, &ui)
                {
                    cb.invoke(sample);
                }
            })?);
        }

        if let Some(cb) = input.on_pointer_move {
            let ui = ui_for_events.clone();
            revokers.borrow_mut().push(iue.PointerMoved(move |_sender, args| {
                if let Some(args) = args.as_ref()
                    && let Some(sample) = panel_pointer_sample(args, &ui)
                {
                    cb.invoke(sample);
                }
            })?);
        }

        if let Some(cb) = input.on_pointer_enter {
            let ui = ui_for_events.clone();
            revokers.borrow_mut().push(iue.PointerEntered(move |_sender, args| {
                if let Some(args) = args.as_ref()
                    && let Some(sample) = panel_pointer_sample(args, &ui)
                {
                    cb.invoke(sample);
                }
            })?);
        }

        if let Some(cb) = input.on_pointer_exit {
            revokers
                .borrow_mut()
                .push(iue.PointerExited(move |_sender, _args| {
                    cb.invoke(());
                })?);
        }

        if let Some(cb) = input.on_wheel {
            let ui = ui_for_events;
            revokers.borrow_mut().push(iue.PointerWheelChanged(move |_sender, args| {
                if let Some(args) = args.as_ref()
                    && let Some(sample) = panel_wheel_sample(args, &ui)
                {
                    cb.invoke(sample);
                }
            })?);
        }

        // Subscriptions must outlive this call; the panel has no explicit
        // teardown hook to revoke them from, so leak them intentionally.
        std::mem::forget(revokers);
        Ok(())
    }
}

/// Widget that hosts custom Direct3D / Direct2D rendering inside WinUI.
#[derive(Clone, Debug, PartialEq)]
pub struct SwapChainPanel {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub mounted: Option<Callback<Option<windows_core::IInspectable>>>,
    pub unmounted: Option<Callback<Option<windows_core::IInspectable>>>,
}

impl ElementRefExt for SwapChainPanel {
    type Handle = SwapChainPanelHandle;
}

impl Default for SwapChainPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SwapChainPanel {
    pub fn new() -> Self {
        Self {
            key: None,
            modifiers: Modifiers::default(),
            mounted: None,
            unmounted: None,
        }
    }

    /// Callback invoked once after the native control is created.
    pub fn on_mounted(mut self, f: impl Fn(SwapChainPanelHandle) + 'static) -> Self {
        self.mounted = Some(Callback::new(move |native: Option<_>| {
            if let Some(native) = native {
                f(SwapChainPanelHandle(native));
            }
        }));
        self
    }

    /// Callback invoked just before the native control is destroyed.
    pub fn on_unmounted(mut self, f: impl Fn(SwapChainPanelHandle) + 'static) -> Self {
        self.unmounted = Some(Callback::new(move |native: Option<_>| {
            if let Some(native) = native {
                f(SwapChainPanelHandle(native));
            }
        }));
        self
    }

    /// Callback invoked when the panel's layout size changes.
    pub fn on_resize(mut self, f: impl Fn(f64, f64) + 'static) -> Self {
        let f = Rc::new(f);
        let prev = self.mounted.take();
        self.mounted = Some(Callback::new(
            move |native: Option<windows_core::IInspectable>| {
                if let Some(ref cb) = prev {
                    cb.invoke(native.clone());
                }
                let Some(native) = native else {
                    return;
                };
                if let Ok(fe) = native.cast::<bindings::IFrameworkElement>() {
                    let f_for_changed = f.clone();
                    if let Ok(revoker) = fe.SizeChanged(move |_sender, args| {
                        if let Some(args) = args.as_ref()
                            && let Ok(s) = args.NewSize()
                        {
                            f_for_changed(s.width as f64, s.height as f64);
                        }
                    }) {
                        // `into_token` avoids pinning the element alive forever.
                        let _ = revoker.into_token();
                    }
                    // If layout already ran before the subscription was
                    // added, `SizeChanged` will not replay — seed the
                    // callback once so first-show paints are not skipped.
                    if let Ok(w) = fe.ActualWidth()
                        && let Ok(h) = fe.ActualHeight()
                        && w > 0.0
                        && h > 0.0
                    {
                        f(w, h);
                    }
                }
            },
        ));
        self
    }
}

impl Widget for SwapChainPanel {
    widget_header!(ControlKind::SwapChainPanel);
    fn bindings(&self) -> PropBindings {
        Vec::new()
    }
    fn on_mounted_callback(&self) -> Option<&Callback<Option<windows_core::IInspectable>>> {
        self.mounted.as_ref()
    }
    fn on_unmounted_callback(&self) -> Option<&Callback<Option<windows_core::IInspectable>>> {
        self.unmounted.as_ref()
    }
}

/// Creates a [`SwapChainPanel`].
pub fn swap_chain_panel() -> SwapChainPanel {
    SwapChainPanel::new()
}

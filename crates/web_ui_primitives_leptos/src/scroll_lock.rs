#[cfg(target_arch = "wasm32")]
use std::sync::{Mutex, OnceLock};

#[cfg(target_arch = "wasm32")]
use web_sys::{CssStyleDeclaration, window};

/// Errors that can occur while applying or restoring scroll locking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollLockError {
    WindowUnavailable,
    DocumentUnavailable,
    BodyUnavailable,
    StyleUnavailable,
}

/// Result type used by scroll lock helpers.
pub type ScrollLockResult<T> = Result<T, ScrollLockError>;

/// RAII guard returned by [`scroll_lock_acquire`].
///
/// Dropping the guard releases one scroll lock reference.
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct ScrollLockGuard {
    active: bool,
}

/// Stateless native guard returned when no browser document exists.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct ScrollLockGuard;

#[cfg(target_arch = "wasm32")]
impl Drop for ScrollLockGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = scroll_lock_release();
            self.active = false;
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Default, Clone)]
struct ScrollLockSnapshot {
    scroll_y: f64,
    overflow: String,
    position: String,
    top: String,
    width: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Default)]
struct ScrollLockState {
    count: usize,
    snapshot: Option<ScrollLockSnapshot>,
}

#[cfg(target_arch = "wasm32")]
static SCROLL_LOCK_STATE: OnceLock<Mutex<ScrollLockState>> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
fn scroll_lock_state() -> &'static Mutex<ScrollLockState> {
    SCROLL_LOCK_STATE.get_or_init(|| Mutex::new(ScrollLockState::default()))
}

/// Acquires a global scroll lock.
///
/// On wasm this snapshots body styles and fixes the body in place until all
/// acquired guards have been released.
#[cfg(target_arch = "wasm32")]
pub fn scroll_lock_acquire() -> ScrollLockResult<ScrollLockGuard> {
    let mut state = scroll_lock_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if state.count == 0 {
        scroll_lock_apply(&mut state)?;
    }
    state.count += 1;
    Ok(ScrollLockGuard { active: true })
}

/// Returns a stateless no-op guard on native targets.
///
/// Native SSR never creates process-global scroll-lock state.
#[cfg(not(target_arch = "wasm32"))]
pub fn scroll_lock_acquire() -> ScrollLockResult<ScrollLockGuard> {
    Ok(ScrollLockGuard)
}

/// Releases one global scroll lock reference.
///
/// Extra releases are ignored. When the final reference is released on wasm,
/// the original body styles and scroll position are restored.
#[cfg(target_arch = "wasm32")]
pub fn scroll_lock_release() -> ScrollLockResult<()> {
    let mut state = scroll_lock_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if state.count == 0 {
        return Ok(());
    }
    state.count = state.count.saturating_sub(1);
    if state.count == 0 {
        scroll_lock_restore(&mut state)?;
    }
    Ok(())
}

/// Performs no work on native targets because no browser lock exists.
#[cfg(not(target_arch = "wasm32"))]
pub fn scroll_lock_release() -> ScrollLockResult<()> {
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn scroll_lock_apply(state: &mut ScrollLockState) -> ScrollLockResult<()> {
    let window = window().ok_or(ScrollLockError::WindowUnavailable)?;
    let document = window
        .document()
        .ok_or(ScrollLockError::DocumentUnavailable)?;
    let body = document.body().ok_or(ScrollLockError::BodyUnavailable)?;
    let style = body.style();
    let scroll_y = window
        .scroll_y()
        .map_err(|_| ScrollLockError::StyleUnavailable)?;
    let snapshot = ScrollLockSnapshot {
        scroll_y,
        overflow: style_value(&style, "overflow")?,
        position: style_value(&style, "position")?,
        top: style_value(&style, "top")?,
        width: style_value(&style, "width")?,
    };
    style_set(&style, "overflow", "hidden")?;
    style_set(&style, "position", "fixed")?;
    style_set(&style, "width", "100%")?;
    style_set(&style, "top", &format!("-{}px", snapshot.scroll_y))?;
    state.snapshot = Some(snapshot);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn scroll_lock_restore(state: &mut ScrollLockState) -> ScrollLockResult<()> {
    let Some(snapshot) = state.snapshot.take() else {
        return Ok(());
    };
    let window = window().ok_or(ScrollLockError::WindowUnavailable)?;
    let document = window
        .document()
        .ok_or(ScrollLockError::DocumentUnavailable)?;
    let body = document.body().ok_or(ScrollLockError::BodyUnavailable)?;
    let style = body.style();
    style_set(&style, "overflow", &snapshot.overflow)?;
    style_set(&style, "position", &snapshot.position)?;
    style_set(&style, "top", &snapshot.top)?;
    style_set(&style, "width", &snapshot.width)?;
    window.scroll_to_with_x_and_y(0.0, snapshot.scroll_y);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn style_set(style: &CssStyleDeclaration, name: &str, value: &str) -> ScrollLockResult<()> {
    style
        .set_property(name, value)
        .map_err(|_| ScrollLockError::StyleUnavailable)
}

#[cfg(target_arch = "wasm32")]
fn style_value(style: &CssStyleDeclaration, name: &str) -> ScrollLockResult<String> {
    style
        .get_property_value(name)
        .map_err(|_| ScrollLockError::StyleUnavailable)
}

#[cfg(all(test, target_arch = "wasm32"))]
fn scroll_lock_count_for_test() -> usize {
    let state = scroll_lock_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.count
}

#[cfg(test)]
mod tests {
    use super::{scroll_lock_acquire, scroll_lock_release};

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn scroll_lock_guard_updates_count() {
        use super::scroll_lock_count_for_test;

        assert_eq!(scroll_lock_count_for_test(), 0);
        let guard = scroll_lock_acquire().expect("lock");
        assert_eq!(scroll_lock_count_for_test(), 1);
        drop(guard);
        assert_eq!(scroll_lock_count_for_test(), 0);
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn scroll_lock_release_is_idempotent() {
        use super::scroll_lock_count_for_test;

        scroll_lock_release().expect("release");
        assert_eq!(scroll_lock_count_for_test(), 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_scroll_lock_guards_are_stateless_across_requests() {
        use super::ScrollLockGuard;

        assert_eq!(core::mem::size_of::<ScrollLockGuard>(), 0);
        std::thread::scope(|scope| {
            for _ in 0..16 {
                scope.spawn(|| {
                    let _guard = scroll_lock_acquire().expect("lock");
                    scroll_lock_release().expect("release");
                });
            }
        });
    }
}

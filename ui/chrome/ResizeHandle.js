import { jsx } from "react/jsx-runtime";
import { useRef } from "react";
const RESIZE_STEP = 16;
function ResizeHandle({
  className,
  orientation,
  label,
  value,
  min,
  max,
  getValue,
  onChange,
  growDirection = -1
}) {
  const dragRef = useRef(null);
  const axis = orientation === "vertical" ? "x" : "y";
  function clamp(next) {
    return Math.max(min, Math.min(max, next));
  }
  function currentValue() {
    return value ?? getValue();
  }
  function coordinate(event) {
    return axis === "x" ? event.clientX : event.clientY;
  }
  function startResize(event) {
    if (event.button !== 0) return;
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    dragRef.current = {
      pointerId: event.pointerId,
      startPosition: coordinate(event),
      startValue: currentValue()
    };
  }
  function resize(event) {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const delta = coordinate(event) - drag.startPosition;
    onChange(clamp(drag.startValue + delta * growDirection));
  }
  function stopResize(event) {
    if (dragRef.current?.pointerId !== event.pointerId) return;
    dragRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  }
  function resizeWithKeyboard(event) {
    const current = currentValue();
    let next = current;
    if (orientation === "vertical" && event.key === "ArrowLeft") next += RESIZE_STEP;
    else if (orientation === "vertical" && event.key === "ArrowRight") next -= RESIZE_STEP;
    else if (orientation === "horizontal" && event.key === "ArrowUp") next += RESIZE_STEP;
    else if (orientation === "horizontal" && event.key === "ArrowDown") next -= RESIZE_STEP;
    else if (event.key === "PageUp") next += RESIZE_STEP * 4;
    else if (event.key === "PageDown") next -= RESIZE_STEP * 4;
    else if (event.key === "Home") next = min;
    else if (event.key === "End") next = max;
    else return;
    event.preventDefault();
    onChange(clamp(next));
  }
  return /* @__PURE__ */ jsx(
    "div",
    {
      className,
      role: "separator",
      "aria-label": label,
      "aria-orientation": orientation,
      "aria-valuemin": min,
      "aria-valuemax": max,
      "aria-valuenow": Math.round(currentValue()),
      tabIndex: 0,
      onPointerDown: startResize,
      onPointerMove: resize,
      onPointerUp: stopResize,
      onPointerCancel: stopResize,
      onKeyDown: resizeWithKeyboard
    }
  );
}
export {
  ResizeHandle
};

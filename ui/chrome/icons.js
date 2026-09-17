import { createElement } from "react";
function tablerIcon(paths) {
  return function Icon({ size = 18, stroke = 1.6, className }) {
    return createElement(
      "svg",
      {
        xmlns: "http://www.w3.org/2000/svg",
        width: size,
        height: size,
        viewBox: "0 0 24 24",
        fill: "none",
        stroke: "currentColor",
        strokeWidth: stroke,
        strokeLinecap: "round",
        strokeLinejoin: "round",
        className,
        "aria-hidden": true
      },
      ...paths.map((d) => createElement("path", { d, key: d }))
    );
  };
}
const IconChevronLeft = tablerIcon(["M15 6l-6 6l6 6"]);
const IconChevronRight = tablerIcon(["M9 6l6 6l-6 6"]);
const IconChevronDown = tablerIcon(["M6 9l6 6l6 -6"]);
const IconCheck = tablerIcon(["M5 12l5 5l10 -10"]);
const IconCopy = tablerIcon([
  "M7 9.667a2.667 2.667 0 0 1 2.667 -2.667h8.666a2.667 2.667 0 0 1 2.667 2.667v8.666a2.667 2.667 0 0 1 -2.667 2.667h-8.666a2.667 2.667 0 0 1 -2.667 -2.667l0 -8.666",
  "M4.012 16.737a2.005 2.005 0 0 1 -1.012 -1.737v-10c0 -1.1 .9 -2 2 -2h10c.75 0 1.158 .385 1.5 1"
]);
const IconDeviceDesktop = tablerIcon([
  "M3 5a1 1 0 0 1 1 -1h16a1 1 0 0 1 1 1v10a1 1 0 0 1 -1 1h-16a1 1 0 0 1 -1 -1v-10",
  "M7 20h10",
  "M9 16v4",
  "M15 16v4"
]);
const IconDeviceTablet = tablerIcon([
  "M5 4a1 1 0 0 1 1 -1h12a1 1 0 0 1 1 1v16a1 1 0 0 1 -1 1h-12a1 1 0 0 1 -1 -1v-16",
  "M11 17a1 1 0 1 0 2 0a1 1 0 0 0 -2 0"
]);
const IconDeviceMobile = tablerIcon([
  "M6 5a2 2 0 0 1 2 -2h8a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2v-14",
  "M11 4h2",
  "M12 17v.01"
]);
const IconSun = tablerIcon([
  "M8 12a4 4 0 1 0 8 0a4 4 0 1 0 -8 0",
  "M3 12h1m8 -9v1m8 8h1m-9 8v1m-6.4 -15.4l.7 .7m12.1 -.7l-.7 .7m0 11.4l.7 .7m-12.1 -.7l-.7 .7"
]);
const IconMoon = tablerIcon([
  "M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454l0 .008"
]);
const IconMenu2 = tablerIcon(["M4 6l16 0", "M4 12l16 0", "M4 18l16 0"]);
const IconAdjustmentsHorizontal = tablerIcon([
  "M12 6a2 2 0 1 0 4 0a2 2 0 1 0 -4 0",
  "M4 6l8 0",
  "M16 6l4 0",
  "M6 12a2 2 0 1 0 4 0a2 2 0 1 0 -4 0",
  "M4 12l2 0",
  "M10 12l10 0",
  "M15 18a2 2 0 1 0 4 0a2 2 0 1 0 -4 0",
  "M4 18l11 0",
  "M19 18l1 0"
]);
export {
  IconAdjustmentsHorizontal,
  IconCheck,
  IconChevronDown,
  IconChevronLeft,
  IconChevronRight,
  IconCopy,
  IconDeviceDesktop,
  IconDeviceMobile,
  IconDeviceTablet,
  IconMenu2,
  IconMoon,
  IconSun
};

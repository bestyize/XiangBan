import type { GlobalThemeOverrides } from "naive-ui";
export const theme: GlobalThemeOverrides = {
  common: {
    primaryColor: "#2563eb", primaryColorHover: "#3b73ef", primaryColorPressed: "#1d4ed8", primaryColorSuppl: "#2563eb",
    infoColor: "#2563eb", successColor: "#16a085", warningColor: "#d99020", errorColor: "#dc5262",
    textColorBase: "#1e293b", textColor1: "#1e293b", textColor2: "#526176", textColor3: "#8a96a8",
    borderColor: "#e4e9f0", dividerColor: "#edf0f5", bodyColor: "#f5f7fb", cardColor: "#ffffff",
    borderRadius: "10px", borderRadiusSmall: "7px", fontSize: "14px",
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Microsoft YaHei UI", sans-serif',
  },
  Button: { fontWeight: "600", heightMedium: "38px", heightLarge: "42px", borderRadiusMedium: "9px", borderRadiusLarge: "10px" },
  Input: { heightMedium: "38px" },
  Select: { peers: { InternalSelection: { heightMedium: "38px" } } },
  Card: { paddingMedium: "24px", titleFontSizeMedium: "16px" },
};

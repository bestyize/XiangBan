// @vitest-environment happy-dom
import { mount, flushPromises, type VueWrapper } from "@vue/test-utils";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { reactive } from "vue";
import LoginModal from "./LoginModal.vue";
import type { ControlStore } from "../control-types.ts";
import { AccountError, defaults, type AccountSession } from "../../bridge.ts";

const session = (username: string): AccountSession => ({
  accessToken: "token123",
  refreshToken: "refresh123",
  expiresAt: 1234567890,
  user: { uid: "uid-1", username },
});

function createStore() {
  const store = reactive({
    desktop: false,
    settings: JSON.parse(JSON.stringify(defaults)),
    library: { items: [], source: "", activeId: "" },
    account: null,
    showLoginModal: true,
    openLogin: vi.fn(() => {
      store.showLoginModal = true;
    }),
    closeLogin: vi.fn(() => {
      store.showLoginModal = false;
    }),
    sendAccountCode: vi.fn().mockResolvedValue({ ok: true, cooldownSeconds: 60, expiresIn: 300 }),
    loginAccountWithCode: vi.fn().mockResolvedValue({ kind: "session", isNewUser: false, session: session("Alice_01") }),
    loginAccountWithPassword: vi.fn().mockResolvedValue({ isNewUser: false, session: session("Alice_01") }),
    registerAccount: vi.fn().mockResolvedValue({ isNewUser: true, session: session("Brand_New") }),
    resetAccountPassword: vi.fn().mockResolvedValue(undefined),
    checkAccountUsername: vi.fn().mockResolvedValue({ available: true }),
    getSignupMethods: vi.fn().mockResolvedValue({ email: true, phone: false }),
    logoutAccount: vi.fn().mockResolvedValue(undefined),
    notify: vi.fn(),
  }) as unknown as ControlStore;
  return store;
}

const q = <T extends Element = HTMLElement>(testid: string) => document.body.querySelector(`[data-testid="${testid}"]`) as T | null;

async function type(testid: string, value: string) {
  const input = q<HTMLInputElement>(testid);
  if (!input) throw new Error(`missing ${testid}`);
  input.value = value;
  input.dispatchEvent(new Event("input"));
  await flushPromises();
}

async function click(testid: string) {
  const el = q<HTMLButtonElement>(testid);
  if (!el) throw new Error(`missing ${testid}`);
  el.click();
  await flushPromises();
}

describe("LoginModal component", () => {
  let wrapper: VueWrapper | undefined;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers({ shouldAdvanceTime: true });
  });

  afterEach(() => {
    wrapper?.unmount();
    wrapper = undefined;
    vi.useRealTimers();
  });

  async function open(store = createStore()) {
    wrapper = mount(LoginModal, { props: { store }, attachTo: document.body });
    await flushPromises();
    return store;
  }

  async function reachAccountSetup(store: ControlStore) {
    vi.mocked(store.loginAccountWithCode).mockResolvedValueOnce({ kind: "registration", registrationToken: "reg-token" });
    await open(store);
    await click("tab-code");
    await type("login-identifier-input", "new@example.com");
    await click("send-code-btn");
    await type("login-code-input", "654321");
    await click("login-submit-btn");
  }

  async function settleUsernameCheck() {
    await vi.advanceTimersByTimeAsync(400);
    await flushPromises();
  }

  it("does not render while the modal is closed", () => {
    const store = createStore();
    store.showLoginModal = false;
    wrapper = mount(LoginModal, { props: { store } });
    expect(q("login-modal-overlay")).toBeNull();
  });

  it("opens on password login and closes from the close button", async () => {
    const store = await open();
    expect(q("login-modal-overlay")?.textContent).toContain("登录小伴 / 快速注册");
    expect(q("login-password-input")).not.toBeNull();
    expect(q("login-code-input")).toBeNull();
    await click("login-close-btn");
    expect(store.closeLogin).toHaveBeenCalledOnce();
  });

  it("logs in with a username and password", async () => {
    const store = await open();
    expect(q<HTMLButtonElement>("login-submit-btn")?.disabled).toBe(true);
    await type("login-identifier-input", "alice_01");
    await type("login-password-input", "pass-word-123");
    await click("login-submit-btn");
    expect(store.loginAccountWithPassword).toHaveBeenCalledWith("alice_01", "pass-word-123");
    expect(store.notify).toHaveBeenCalledWith("欢迎回来，Alice_01", "success");
    expect(store.closeLogin).toHaveBeenCalled();
  });

  it("shows the server's message when the password is wrong and stays open", async () => {
    const store = createStore();
    vi.mocked(store.loginAccountWithPassword).mockRejectedValueOnce(new AccountError("账号或密码错误", null));
    await open(store);
    await type("login-identifier-input", "alice_01");
    await type("login-password-input", "wrong-password");
    await click("login-submit-btn");
    expect(q("login-error-msg")?.textContent).toContain("账号或密码错误");
    expect(store.closeLogin).not.toHaveBeenCalled();
  });

  it("sends a code, shows the countdown, then signs an existing account in", async () => {
    const store = await open();
    await click("tab-code");
    expect(q<HTMLButtonElement>("send-code-btn")?.disabled).toBe(true);
    await type("login-identifier-input", "user@example.com");
    expect(q<HTMLButtonElement>("send-code-btn")?.disabled).toBe(false);
    await click("send-code-btn");
    expect(store.sendAccountCode).toHaveBeenCalledWith("user@example.com", "login");
    expect(q("send-code-btn")?.textContent).toContain("60s 后重发");

    await type("login-code-input", "123456");
    await click("login-submit-btn");
    expect(store.loginAccountWithCode).toHaveBeenCalledWith("user@example.com", "123456");
    expect(store.notify).toHaveBeenCalledWith("欢迎回来，Alice_01", "success");
    expect(store.registerAccount).not.toHaveBeenCalled();
  });

  it("walks a new email through account setup with a live username check", async () => {
    const store = createStore();
    await reachAccountSetup(store);

    expect(q("login-subtitle")?.textContent).toContain("设置后不能修改");
    expect(q("login-username-input")).not.toBeNull();
    expect(q<HTMLButtonElement>("login-submit-btn")?.disabled).toBe(true);

    await type("login-username-input", "1bad");
    expect(q("username-hint")?.textContent).toContain("字母或汉字开头");
    expect(store.checkAccountUsername).not.toHaveBeenCalled();

    await type("login-username-input", "Brand_New");
    expect(q("username-hint")?.textContent).toContain("正在检查");
    await settleUsernameCheck();
    expect(store.checkAccountUsername).toHaveBeenCalledWith("Brand_New");
    expect(q("username-hint")?.textContent).toContain("可以使用");

    await type("login-password-input", "short");
    expect(q("password-hint")?.textContent).toContain("至少 8 位");
    await type("login-password-input", "pass-word-123");
    await click("login-submit-btn");

    expect(store.registerAccount).toHaveBeenCalledWith("reg-token", "Brand_New", "pass-word-123");
    expect(store.notify).toHaveBeenCalledWith("注册成功，欢迎你，Brand_New", "success");
    expect(store.closeLogin).toHaveBeenCalled();
  });

  it("offers email only for code sign-up until the account center opens phone numbers", async () => {
    await open();
    await click("tab-code");
    expect(q("login-identifier-input")?.getAttribute("placeholder")).toBe("name@example.com");
    expect(document.body.querySelector('label[for="login-identifier"]')?.textContent).toBe("邮箱");

    await type("login-identifier-input", "13800138000");
    await click("send-code-btn");
    expect(q("login-error-msg")?.textContent).toContain("暂未开放手机号注册");
  });

  it("keeps phone numbers for password login and shows them for code sign-up when opened", async () => {
    const store = createStore();
    vi.mocked(store.getSignupMethods).mockResolvedValue({ email: true, phone: true });
    await open(store);
    expect(document.body.querySelector('label[for="login-identifier"]')?.textContent).toContain("手机号");
    await click("tab-code");
    expect(document.body.querySelector('label[for="login-identifier"]')?.textContent).toBe("邮箱或手机号");
  });

  it("falls back to email only when the methods cannot be read", async () => {
    const store = createStore();
    vi.mocked(store.getSignupMethods).mockRejectedValue(new Error("offline"));
    await open(store);
    await click("tab-code");
    expect(document.body.querySelector('label[for="login-identifier"]')?.textContent).toBe("邮箱");
  });

  it("blocks a username the server says is taken", async () => {
    const store = createStore();
    vi.mocked(store.checkAccountUsername).mockResolvedValueOnce({ available: false, message: "这个用户名已被占用，换一个吧" });
    await reachAccountSetup(store);

    await type("login-username-input", "Taken_Name");
    await settleUsernameCheck();
    expect(q("username-hint")?.textContent).toContain("已被占用");
    await type("login-password-input", "pass-word-123");
    expect(q<HTMLButtonElement>("login-submit-btn")?.disabled).toBe(true);
  });

  it("returns to the start when the registration token has expired", async () => {
    const store = createStore();
    vi.mocked(store.registerAccount).mockRejectedValueOnce(new AccountError("验证已过期，请重新获取验证码", "registration_expired"));
    await reachAccountSetup(store);
    await type("login-username-input", "Brand_New");
    await settleUsernameCheck();
    await type("login-password-input", "pass-word-123");
    await click("login-submit-btn");

    expect(q("login-error-msg")?.textContent).toContain("验证已过期");
    expect(q("login-identifier-input")).not.toBeNull();
    expect(q("login-username-input")).toBeNull();
  });

  it("resets a forgotten password with a code and goes back to password login", async () => {
    const store = await open();
    await click("forgot-password");
    await type("login-identifier-input", "me@example.com");
    await click("send-code-btn");
    expect(store.sendAccountCode).toHaveBeenCalledWith("me@example.com", "reset_password");
    await type("login-code-input", "111222");
    await type("login-password-input", "brand-new-pass-9");
    await click("login-submit-btn");

    expect(store.resetAccountPassword).toHaveBeenCalledWith("me@example.com", "111222", "brand-new-pass-9");
    expect(q("login-notice")?.textContent).toContain("密码已重置");
    expect(q("tab-password")?.getAttribute("aria-selected")).toBe("true");
  });
});

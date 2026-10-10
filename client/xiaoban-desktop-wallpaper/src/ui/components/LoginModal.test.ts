// @vitest-environment happy-dom
import { mount, flushPromises } from "@vue/test-utils";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { reactive } from "vue";
import LoginModal from "./LoginModal.vue";
import type { ControlStore } from "../control-types.ts";
import { defaults } from "../../bridge.ts";

function createStore() {
  const store = reactive({
    desktop: false,
    settings: JSON.parse(JSON.stringify(defaults)),
    library: { items: [], source: "", activeId: "" },
    account: null,
    showLoginModal: false,
    openLogin: vi.fn(() => {
      store.showLoginModal = true;
    }),
    closeLogin: vi.fn(() => {
      store.showLoginModal = false;
    }),
    sendAccountCode: vi.fn().mockResolvedValue({ ok: true, cooldownSeconds: 60, expiresIn: 300 }),
    loginAccount: vi.fn().mockImplementation((email: string) => {
      const isNew = email.includes("new");
      return Promise.resolve({
        isNewUser: isNew,
        session: {
          accessToken: "token123",
          refreshToken: "refresh123",
          expiresAt: 1234567890,
          user: {
            uid: "uid-1",
            nickname: "测试小伴",
            email,
          },
        },
      });
    }),
    logoutAccount: vi.fn().mockResolvedValue(undefined),
    notify: vi.fn(),
  }) as unknown as ControlStore;
  return store;
}

describe("LoginModal component", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not render when showLoginModal is false", () => {
    const store = createStore();
    mount(LoginModal, { props: { store } });
    expect(document.body.querySelector('[data-testid="login-modal-overlay"]')).toBeNull();
  });

  it("renders modal when showLoginModal is true and closes on close button click", async () => {
    const store = createStore();
    store.showLoginModal = true;
    const wrapper = mount(LoginModal, { props: { store } });
    await flushPromises();

    const overlay = document.body.querySelector('[data-testid="login-modal-overlay"]');
    expect(overlay).not.toBeNull();
    expect(overlay?.textContent).toContain("登录小伴 / 快速注册");
    expect(overlay?.textContent).toContain("未注册邮箱验证后自动创建账号，已注册直接登录");

    const closeBtn = document.body.querySelector('[data-testid="login-close-btn"]') as HTMLButtonElement;
    expect(closeBtn).not.toBeNull();
    closeBtn.click();
    await flushPromises();

    expect(store.closeLogin).toHaveBeenCalledOnce();
    wrapper.unmount();
  });

  it("sends verification code on valid email and shows countdown", async () => {
    const store = createStore();
    store.showLoginModal = true;
    const wrapper = mount(LoginModal, { props: { store } });
    await flushPromises();

    const emailInput = document.body.querySelector('[data-testid="login-email-input"]') as HTMLInputElement;
    const sendBtn = document.body.querySelector('[data-testid="send-code-btn"]') as HTMLButtonElement;

    // Disabled initially
    expect(sendBtn.disabled).toBe(true);

    // Set invalid email
    emailInput.value = "notanemail";
    emailInput.dispatchEvent(new Event("input"));
    await flushPromises();
    expect(sendBtn.disabled).toBe(true);

    // Set valid email
    emailInput.value = "user@example.com";
    emailInput.dispatchEvent(new Event("input"));
    await flushPromises();
    expect(sendBtn.disabled).toBe(false);

    // Click send code
    sendBtn.click();
    await flushPromises();

    expect(store.sendAccountCode).toHaveBeenCalledWith("user@example.com");
    expect(store.notify).toHaveBeenCalledWith("验证码已发送，请查收邮件", "success");
    expect(sendBtn.textContent).toContain("60s 后重发");

    wrapper.unmount();
  });

  it("logs in successfully for existing user", async () => {
    const store = createStore();
    store.showLoginModal = true;
    const wrapper = mount(LoginModal, { props: { store }, attachTo: document.body });
    await flushPromises();

    const emailInput = document.body.querySelector('[data-testid="login-email-input"]') as HTMLInputElement;
    const codeInput = document.body.querySelector('[data-testid="login-code-input"]') as HTMLInputElement;
    const submitBtn = document.body.querySelector('[data-testid="login-submit-btn"]') as HTMLButtonElement;

    emailInput.value = "existing@example.com";
    emailInput.dispatchEvent(new Event("input"));
    codeInput.value = "123456";
    codeInput.dispatchEvent(new Event("input"));
    await flushPromises();

    submitBtn.click();
    await flushPromises();

    expect(store.loginAccount).toHaveBeenCalledWith("existing@example.com", "123456");
    expect(store.notify).toHaveBeenCalledWith("欢迎回来，测试小伴！", "success");
    expect(store.closeLogin).toHaveBeenCalled();

    wrapper.unmount();
  });

  it("creates account and logs in for new user", async () => {
    const store = createStore();
    store.showLoginModal = true;
    const wrapper = mount(LoginModal, { props: { store }, attachTo: document.body });
    await flushPromises();

    const emailInput = document.body.querySelector('[data-testid="login-email-input"]') as HTMLInputElement;
    const codeInput = document.body.querySelector('[data-testid="login-code-input"]') as HTMLInputElement;
    const submitBtn = document.body.querySelector('[data-testid="login-submit-btn"]') as HTMLButtonElement;

    emailInput.value = "newuser@example.com";
    emailInput.dispatchEvent(new Event("input"));
    codeInput.value = "654321";
    codeInput.dispatchEvent(new Event("input"));
    await flushPromises();

    submitBtn.click();
    await flushPromises();

    expect(store.loginAccount).toHaveBeenCalledWith("newuser@example.com", "654321");
    expect(store.notify).toHaveBeenCalledWith("欢迎新朋友！已为你自动创建账号并登录", "success");
    expect(store.closeLogin).toHaveBeenCalled();

    wrapper.unmount();
  });

  it("displays server error message when login fails", async () => {
    const store = createStore();
    vi.mocked(store.loginAccount).mockRejectedValueOnce(new Error("验证码错误或已失效"));
    store.showLoginModal = true;
    const wrapper = mount(LoginModal, { props: { store }, attachTo: document.body });
    await flushPromises();

    const emailInput = document.body.querySelector('[data-testid="login-email-input"]') as HTMLInputElement;
    const codeInput = document.body.querySelector('[data-testid="login-code-input"]') as HTMLInputElement;
    const submitBtn = document.body.querySelector('[data-testid="login-submit-btn"]') as HTMLButtonElement;

    emailInput.value = "err@example.com";
    emailInput.dispatchEvent(new Event("input"));
    codeInput.value = "000000";
    codeInput.dispatchEvent(new Event("input"));
    await flushPromises();

    submitBtn.click();
    await flushPromises();

    const errorBanner = document.body.querySelector('[data-testid="login-error-msg"]');
    expect(errorBanner).not.toBeNull();
    expect(errorBanner?.textContent).toContain("验证码错误或已失效");
    expect(store.closeLogin).not.toHaveBeenCalled();

    wrapper.unmount();
  });
});

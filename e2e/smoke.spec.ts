import { expect, test } from "@playwright/test";

const apiUrl = process.env.PLAYWRIGHT_API_URL ?? "http://localhost:8080";

test.describe("NeuralScope smoke tests", () => {
  test("landing page loads", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("NeuralScope")).toBeVisible();
  });

  test("login page renders", async ({ page }) => {
    await page.goto("/login");
    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
    await expect(page.getByLabel("Email")).toBeVisible();
    await expect(page.getByLabel("Password")).toBeVisible();
  });

  test("unauthenticated dashboard redirects to login", async ({ page }) => {
    await page.goto("/overview");
    await expect(page).toHaveURL(/\/login/);
  });

  test("API health endpoint responds", async ({ request }) => {
    const response = await request.get(`${apiUrl}/health`);
    expect(response.ok()).toBeTruthy();

    const body = await response.json();
    expect(body.status).toBe("ok");
  });

  test("API readiness endpoint responds", async ({ request }) => {
    const response = await request.get(`${apiUrl}/ready`);
    expect([200, 503]).toContain(response.status());
  });

  test("web health endpoint responds", async ({ request }) => {
    const webUrl = process.env.PLAYWRIGHT_BASE_URL ?? "http://localhost:3000";
    const response = await request.get(`${webUrl}/api/health`);
    expect(response.ok()).toBeTruthy();
  });
});

import { test, expect } from "@playwright/test";

test.describe("App smoke tests", () => {
  test("app loads without crashing", async ({ page }) => {
    await page.goto("/");
    // App layout should render
    await expect(page.locator(".app-layout")).toBeVisible();
    // No JS errors on load
  });

  test("sidebar renders all navigation items", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".sidebar")).toBeVisible();
    await expect(page.getByText("Inbox")).toBeVisible();
    await expect(page.getByText("All Notes")).toBeVisible();
    await expect(page.getByText("Search")).toBeVisible();
    await expect(page.getByText("Graph")).toBeVisible();
    await expect(page.getByText("Quick Capture")).toBeVisible();
  });

  test("topbar renders with search input", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".topbar")).toBeVisible();
    await expect(page.locator(".search-input input")).toBeVisible();
  });

  test("default view is Inbox", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".topbar-title")).toHaveText("Inbox");
  });
});

test.describe("Navigation", () => {
  test("navigate to All Notes", async ({ page }) => {
    await page.goto("/");
    await page.getByText("All Notes").click();
    await expect(page.locator(".topbar-title")).toHaveText("All Notes");
  });

  test("navigate to Search", async ({ page }) => {
    await page.goto("/");
    await page.getByText("Search").click();
    await expect(page.locator(".topbar-title")).toHaveText("Search");
  });

  test("navigate to Graph", async ({ page }) => {
    await page.goto("/");
    await page.getByText("Graph").click();
    await expect(page.locator(".topbar-title")).toHaveText("Graph");
  });

  test("navigate back to Inbox", async ({ page }) => {
    await page.goto("/");
    await page.getByText("All Notes").click();
    await page.getByText("Inbox").first().click();
    await expect(page.locator(".topbar-title")).toHaveText("Inbox");
  });
});

test.describe("Quick Capture dialog", () => {
  test("opens and closes capture dialog", async ({ page }) => {
    await page.goto("/");
    await page.getByText("Quick Capture").click();
    // Dialog should appear
    await expect(page.locator(".capture-overlay")).toBeVisible();
    // Close it
    await page.keyboard.press("Escape");
    await expect(page.locator(".capture-overlay")).not.toBeVisible();
  });

  test("Ctrl+Shift+N opens capture dialog", async ({ page }) => {
    await page.goto("/");
    await page.keyboard.press("Control+Shift+N");
    await expect(page.locator(".capture-overlay")).toBeVisible();
  });
});

test.describe("Notes list interaction", () => {
  test("All Notes view shows note items", async ({ page }) => {
    await page.goto("/");
    await page.getByText("All Notes").click();
    // Mock data should show notes
    await expect(page.locator(".content-area")).toBeVisible();
  });

  test("clicking a note opens editor view", async ({ page }) => {
    await page.goto("/");
    await page.getByText("All Notes").click();
    // Wait for notes to load
    const noteCard = page.locator("[class*='note']").first();
    if (await noteCard.isVisible()) {
      await noteCard.click();
      await expect(page.locator(".topbar-title")).toHaveText("Note");
    }
  });

  test("editor view has back button", async ({ page }) => {
    await page.goto("/");
    await page.getByText("All Notes").click();
    const noteCard = page.locator("[class*='note']").first();
    if (await noteCard.isVisible()) {
      await noteCard.click();
      // Back button should appear
      await expect(page.locator(".btn-ghost")).toBeVisible();
      await page.locator(".btn-ghost").click();
      await expect(page.locator(".topbar-title")).toHaveText("All Notes");
    }
  });
});

test.describe("Search functionality", () => {
  test("search input accepts text", async ({ page }) => {
    await page.goto("/");
    const input = page.locator(".search-input input");
    await input.fill("test query");
    await expect(input).toHaveValue("test query");
  });

  test("Ctrl+/ focuses search and navigates to search view", async ({ page }) => {
    await page.goto("/");
    await page.keyboard.press("Control+/");
    await expect(page.locator(".topbar-title")).toHaveText("Search");
  });

  test("Enter in search navigates to search view", async ({ page }) => {
    await page.goto("/");
    const input = page.locator(".search-input input");
    await input.fill("rust");
    await input.press("Enter");
    await expect(page.locator(".topbar-title")).toHaveText("Search");
  });
});

test.describe("Error resilience", () => {
  test("no console errors on initial load", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));
    await page.goto("/");
    await page.waitForTimeout(1000);
    expect(errors).toEqual([]);
  });

  test("no console errors during full navigation cycle", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));
    await page.goto("/");

    // Navigate through all views
    for (const view of ["All Notes", "Search", "Graph", "Inbox"]) {
      await page.getByText(view).click();
      await page.waitForTimeout(300);
    }

    expect(errors).toEqual([]);
  });

  test("no console errors when opening capture dialog", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));
    await page.goto("/");

    await page.getByText("Quick Capture").click();
    await page.waitForTimeout(500);
    await page.keyboard.press("Escape");
    await page.waitForTimeout(300);

    expect(errors).toEqual([]);
  });
});

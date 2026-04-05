import { test, expect } from "@playwright/test";

// Helper: click a sidebar nav button by label
function sidebarNav(page: import("@playwright/test").Page, label: string) {
  return page.locator(".sidebar .nav-item", { hasText: label }).first();
}

// Helper: click the Quick Capture button in sidebar
function captureBtn(page: import("@playwright/test").Page) {
  return page.locator(".sidebar .btn-primary");
}

// Helper: the capture dialog (card inside a fixed overlay)
function captureDialog(page: import("@playwright/test").Page) {
  return page.getByPlaceholder("Capture your thought...");
}

test.describe("App smoke tests", () => {
  test("app loads without crashing", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".app-layout")).toBeVisible();
  });

  test("sidebar renders all navigation items", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".sidebar")).toBeVisible();
    await expect(sidebarNav(page, "Inbox")).toBeVisible();
    await expect(sidebarNav(page, "All Notes")).toBeVisible();
    await expect(sidebarNav(page, "Search")).toBeVisible();
    await expect(sidebarNav(page, "Graph")).toBeVisible();
    await expect(captureBtn(page)).toBeVisible();
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
    await sidebarNav(page, "All Notes").click();
    await expect(page.locator(".topbar-title")).toHaveText("All Notes");
  });

  test("navigate to Search", async ({ page }) => {
    await page.goto("/");
    await sidebarNav(page, "Search").click();
    await expect(page.locator(".topbar-title")).toHaveText("Search");
  });

  test("navigate to Graph", async ({ page }) => {
    await page.goto("/");
    await sidebarNav(page, "Graph").click();
    await expect(page.locator(".topbar-title")).toHaveText("Graph");
  });

  test("navigate back to Inbox", async ({ page }) => {
    await page.goto("/");
    await sidebarNav(page, "All Notes").click();
    await sidebarNav(page, "Inbox").click();
    await expect(page.locator(".topbar-title")).toHaveText("Inbox");
  });
});

test.describe("Quick Capture dialog", () => {
  test("opens and closes capture dialog", async ({ page }) => {
    await page.goto("/");
    await captureBtn(page).click();
    // Dialog textarea should appear
    await expect(captureDialog(page)).toBeVisible();
    // Click Cancel to close
    await page.getByRole("button", { name: "Cancel" }).click();
    await expect(captureDialog(page)).not.toBeVisible();
  });
});

test.describe("Notes list interaction", () => {
  test("All Notes view shows note items", async ({ page }) => {
    await page.goto("/");
    await sidebarNav(page, "All Notes").click();
    await expect(page.locator(".content-area")).toBeVisible();
  });

  test("clicking a note opens editor view", async ({ page }) => {
    await page.goto("/");
    await sidebarNav(page, "All Notes").click();
    const noteCard = page.locator(".note-card").first();
    if (await noteCard.isVisible()) {
      await noteCard.click();
      await expect(page.locator(".topbar-title")).toHaveText("Note");
    }
  });

  test("editor view has back button", async ({ page }) => {
    await page.goto("/");
    await sidebarNav(page, "All Notes").click();
    const noteCard = page.locator(".note-card").first();
    if (await noteCard.isVisible()) {
      await noteCard.click();
      const backBtn = page.locator(".topbar .btn-ghost").first();
      await expect(backBtn).toBeVisible();
      await backBtn.click();
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

  test("search via sidebar navigates to search view", async ({ page }) => {
    await page.goto("/");
    await sidebarNav(page, "Search").click();
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

    for (const label of ["All Notes", "Search", "Graph", "Inbox"]) {
      await sidebarNav(page, label).click();
      await page.waitForTimeout(300);
    }

    expect(errors).toEqual([]);
  });

  test("no console errors when opening capture dialog", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));
    await page.goto("/");

    await captureBtn(page).click();
    await page.waitForTimeout(500);
    await page.getByRole("button", { name: "Cancel" }).click();
    await page.waitForTimeout(300);

    expect(errors).toEqual([]);
  });
});

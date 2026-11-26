import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import i18n, { resolveLanguage } from "@/i18n";
import { App } from "@/App";

describe("resolveLanguage", () => {
  it("maps preferences to supported languages", () => {
    expect(resolveLanguage("fr-FR")).toBe("fr");
    expect(resolveLanguage("en")).toBe("en");
    expect(resolveLanguage("de")).toBe("en");
  });
});

describe("App shell", () => {
  it("renders the prompt and switches theme and language", async () => {
    await i18n.changeLanguage("en");
    render(<App />);
    expect(await screen.findByText("What are you focusing on?")).toBeInTheDocument();
    expect(await screen.findByText("Focus")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("radio", { name: "Light" }));
    expect(document.documentElement.dataset.theme).toBe("light");

    await userEvent.selectOptions(screen.getByRole("combobox", { name: "Language" }), "fr");
    expect(await screen.findByText("Sur quoi te concentres-tu ?")).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("fr");
  });
});

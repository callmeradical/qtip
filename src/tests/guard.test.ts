import { Guard } from "../guard";

describe("Guard", () => {
  it("should be initializable", () => {
    const guard = new Guard();
    expect(guard).toBeDefined();
  });

  it("should fail when starting (as expected in Red phase)", async () => {
    const guard = new Guard();
    await expect(guard.start()).rejects.toThrow("Not implemented");
  });
});

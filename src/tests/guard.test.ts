import { Guard } from "../guard";

describe("Guard", () => {
  it("should be initializable with a config path", () => {
    const guard = new Guard("qtip-guard.yaml");
    expect(guard).toBeDefined();
  });

  it("should fail when starting with non-existent config (as expected)", async () => {
    const guard = new Guard("non-existent.yaml");
    expect(() => guard.start()).toThrow();
  });
});

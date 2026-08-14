import { describe, expect, it } from "bun:test";
import { assemble, Hangul } from "../index";

describe("node bindings", () => {
	it("exposes Hangul methods", () => {
		const hangul = new Hangul("가A값!");

		expect(hangul.length).toBe(4);
		expect(hangul.disassemble()).toBe("ㄱㅏAㄱㅏㅂㅅ!");
		expect(hangul.getChoseong()).toBe("ㄱAㄱ!");
		expect(hangul.hasBatchim()).toBe(true);
		expect(hangul.josa("을/를")).toBe("가A값을!");
		expect(hangul.josaParticle("을/를")).toBe("을");
		expect(hangul.containsChoseong("ㄱA")).toBe(true);
		expect(hangul.findChoseong("ㄱA")?.end).toBe(2);
		expect(hangul.get(0)?.isHangul).toBe(true);
		expect(hangul.get(1)?.original).toBe("A");
		expect(hangul.get(4)).toBeNull();
		expect(hangul.disassembleToGroups()).toEqual([
			["ㄱ", "ㅏ"],
			["A"],
			["ㄱ", "ㅏ", "ㅂ", "ㅅ"],
			["!"],
		]);
		expect(() => new Hangul("사과").josa("을")).toThrow();
	});

	it("assembles jamo through the free function", () => {
		expect(assemble("ㄱㅏㅂㅅ")).toBe("값");
		expect(assemble("ㄱㅏㄱㅅㅏ")).toBe("각사");
		expect(assemble("ㄱㅏㄱㅅㅏ", "compound-final")).toBe("갃ㅏ");
		expect(() => assemble("ㄱㅏ", "unknown" as "next-syllable")).toThrow();
	});
});

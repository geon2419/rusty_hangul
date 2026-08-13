import { describe, it, expect } from "vitest";
import { assemble, Hangul } from "../index";

describe("Hangul class", () => {
	describe("disassemble method", () => {
		// 기본 한글 문자를 올바르게 분해하는지 테스트
		it("should correctly disassemble basic Hangul characters", () => {
			expect(new Hangul("안녕").disassemble()).toBe("ㅇㅏㄴㄴㅕㅇ");
			expect(new Hangul("가나다").disassemble()).toBe("ㄱㅏㄴㅏㄷㅏ");
			expect(new Hangul("한글").disassemble()).toBe("ㅎㅏㄴㄱㅡㄹ");
		});

		// 한글 분해 시 비-한글 문자가 보존되는지 테스트
		it("should preserve non-Hangul characters while disassembling Hangul", () => {
			expect(new Hangul("Hello 안녕!").disassemble()).toBe(
				"Hello ㅇㅏㄴㄴㅕㅇ!",
			);
			expect(new Hangul("123 한글 ABC").disassemble()).toBe(
				"123 ㅎㅏㄴㄱㅡㄹ ABC",
			);
		});

		// 빈 문자열이 주어졌을 때 빈 문자열을 반환하는지 테스트
		it("should return an empty string when given an empty string", () => {
			expect(new Hangul("").disassemble()).toBe("");
		});

		// 복잡한 음절을 올바르게 분해하는지 테스트
		it("should correctly disassemble complex syllables", () => {
			expect(new Hangul("꿈").disassemble()).toBe("ㄲㅜㅁ");
			expect(new Hangul("밝다").disassemble()).toBe("ㅂㅏㄹㄱㄷㅏ");
			expect(new Hangul("닭고기").disassemble()).toBe("ㄷㅏㄹㄱㄱㅗㄱㅣ");
		});

		// 공백이 있는 문자열을 올바르게 처리하는지 테스트
		it("should handle strings with spaces correctly", () => {
			expect(new Hangul("안녕 하세요").disassemble()).toBe(
				"ㅇㅏㄴㄴㅕㅇ ㅎㅏㅅㅔㅇㅛ",
			);
		});

		// 공백/개행/탭이 포함된 문자열을 올바르게 처리하는지 테스트
		it("should preserve whitespace characters", () => {
			expect(new Hangul("안녕\n하세요\t").disassemble()).toBe(
				"ㅇㅏㄴㄴㅕㅇ\nㅎㅏㅅㅔㅇㅛ\t",
			);
		});

		// 비한글만 포함된 문자열을 그대로 유지하는지 테스트
		it("should return the same string when input is non-Hangul only", () => {
			expect(new Hangul("ABC123!@").disassemble()).toBe("ABC123!@");
		});

		// 한글과 비한글이 섞인 경계 위치를 올바르게 처리하는지 테스트
		it("should handle mixed boundaries correctly", () => {
			expect(new Hangul("가A나!").disassemble()).toBe("ㄱㅏAㄴㅏ!");
			expect(new Hangul("A가").disassemble()).toBe("Aㄱㅏ");
			expect(new Hangul("가A").disassemble()).toBe("ㄱㅏA");
		});

		// NFD 입력을 음절 단위로 분해하는지 테스트
		it("should disassemble NFD input", () => {
			const nfd = "\u1100\u1161\u11AB";
			expect(new Hangul(nfd).disassemble()).toBe("ㄱㅏㄴ");
		});

		// 연속 NFD 음절을 올바르게 분해하는지 테스트
		it("should disassemble consecutive NFD syllables", () => {
			const annyeong = "\u110B\u1161\u11AB\u1102\u1167\u11BC";
			expect(new Hangul(annyeong).disassemble()).toBe("ㅇㅏㄴㄴㅕㅇ");
		});

		// NFC와 NFD 입력이 같은 분해 결과를 내는지 테스트
		it("should match NFC disassemble results for NFD input", () => {
			expect(new Hangul("\u1100\u116A").disassemble()).toBe(
				new Hangul("과").disassemble(),
			);
			expect(new Hangul("\u1100\u1161\u11B9").disassemble()).toBe(
				new Hangul("값").disassemble(),
			);
		});

		// 단독 조합형 초성 다음에 NFC 음절이 오면 초성은 유지하는지 테스트
		it("should pass through lone choseong before an NFC syllable", () => {
			expect(new Hangul("\u1100가").disassemble()).toBe("\u1100ㄱㅏ");
		});
	});

	describe("getChoseong method", () => {
		// 기본 한글 문자에서 초성을 올바르게 추출하는지 테스트
		it("should correctly extract choseong from basic Hangul characters", () => {
			expect(new Hangul("안녕").getChoseong()).toBe("ㅇㄴ");
			expect(new Hangul("가나다").getChoseong()).toBe("ㄱㄴㄷ");
			expect(new Hangul("한글").getChoseong()).toBe("ㅎㄱ");
		});

		// 초성 추출 시 비-한글 문자가 보존되는지 테스트
		it("should preserve non-Hangul characters while extracting choseong", () => {
			expect(new Hangul("Hello 안녕!").getChoseong()).toBe("Hello ㅇㄴ!");
			expect(new Hangul("123 한글 ABC").getChoseong()).toBe("123 ㅎㄱ ABC");
		});

		// 빈 문자열이 주어졌을 때 빈 문자열을 반환하는지 테스트
		it("should return an empty string when given an empty string", () => {
			expect(new Hangul("").getChoseong()).toBe("");
		});

		// 복잡한 음절에서 초성을 올바르게 추출하는지 테스트
		it("should correctly extract choseong from complex syllables", () => {
			expect(new Hangul("꿈").getChoseong()).toBe("ㄲ");
			expect(new Hangul("밝다").getChoseong()).toBe("ㅂㄷ");
			expect(new Hangul("닭고기").getChoseong()).toBe("ㄷㄱㄱ");
		});

		// 공백이 있는 문자열을 올바르게 처리하는지 테스트
		it("should handle strings with spaces correctly", () => {
			expect(new Hangul("안녕 하세요").getChoseong()).toBe("ㅇㄴ ㅎㅅㅇ");
		});

		// 공백/개행/탭이 포함된 문자열을 올바르게 처리하는지 테스트
		it("should preserve whitespace characters", () => {
			expect(new Hangul("안녕\n하세요\t").getChoseong()).toBe("ㅇㄴ\nㅎㅅㅇ\t");
		});

		// 비한글만 포함된 문자열을 그대로 유지하는지 테스트
		it("should return the same string when input is non-Hangul only", () => {
			expect(new Hangul("ABC123!@").getChoseong()).toBe("ABC123!@");
		});

		// 한글과 비한글이 섞인 경계 위치를 올바르게 처리하는지 테스트
		it("should handle mixed boundaries correctly", () => {
			expect(new Hangul("가A나!").getChoseong()).toBe("ㄱAㄴ!");
			expect(new Hangul("A가").getChoseong()).toBe("Aㄱ");
			expect(new Hangul("가A").getChoseong()).toBe("ㄱA");
		});

		// NFD 입력에서 초성을 추출하는지 테스트
		it("should extract choseong from NFD input", () => {
			const nfd = "\u1100\u1161\u11AB";
			expect(new Hangul(nfd).getChoseong()).toBe("ㄱ");
		});

		// 연속 NFD 음절에서 초성을 추출하는지 테스트
		it("should extract choseong from consecutive NFD syllables", () => {
			const annyeong = "\u110B\u1161\u11AB\u1102\u1167\u11BC";
			expect(new Hangul(annyeong).getChoseong()).toBe("ㅇㄴ");
		});

		// NFC와 NFD 입력이 같은 초성 결과를 내는지 테스트
		it("should match NFC choseong results for NFD input", () => {
			expect(new Hangul("\u1100\u116A").getChoseong()).toBe(
				new Hangul("과").getChoseong(),
			);
			expect(new Hangul("\u1100\u1161\u11B9").getChoseong()).toBe(
				new Hangul("값").getChoseong(),
			);
		});

		// 동일 인스턴스에서 반복 호출 시 결과가 동일한지 테스트
		it("should return the same result on repeated calls", () => {
			const hangul = new Hangul("안녕 Hello");
			expect(hangul.getChoseong()).toBe("ㅇㄴ Hello");
			expect(hangul.getChoseong()).toBe("ㅇㄴ Hello");
		});
	});

	describe("hasBatchim method", () => {
		it("should detect whether the last Hangul syllable has batchim", () => {
			expect(new Hangul("한").hasBatchim()).toBe(true);
			expect(new Hangul("하").hasBatchim()).toBe(false);
			expect(new Hangul("값!").hasBatchim()).toBe(true);
			expect(new Hangul("Hello").hasBatchim()).toBe(false);
		});

		it("should support NFD input", () => {
			expect(new Hangul("\u1112\u1161\u11AB").hasBatchim()).toBe(true);
		});
	});

	describe("josa method", () => {
		it("should select particles for syllables without batchim", () => {
			const hangul = new Hangul("사과");

			expect(hangul.josa("을/를")).toBe("사과를");
			expect(hangul.josa("이/가")).toBe("사과가");
			expect(hangul.josa("은/는")).toBe("사과는");
			expect(hangul.josa("와/과")).toBe("사과와");
			expect(hangul.josa("으로/로")).toBe("사과로");
			expect(hangul.josa("이에요/예요")).toBe("사과예요");
		});

		it("should select particles for syllables with batchim", () => {
			const hangul = new Hangul("수박");

			expect(hangul.josa("을/를")).toBe("수박을");
			expect(hangul.josa("이/가")).toBe("수박이");
			expect(hangul.josa("은/는")).toBe("수박은");
			expect(hangul.josa("와/과")).toBe("수박과");
			expect(hangul.josa("으로/로")).toBe("수박으로");
			expect(hangul.josa("이에요/예요")).toBe("수박이에요");
		});

		it("should handle the rieul exception and trailing punctuation", () => {
			expect(new Hangul("서울").josa("으로/로")).toBe("서울로");
			expect(new Hangul("값!").josa("을/를")).toBe("값을!");
		});

		it("should support NFD input and reject unsupported pairs", () => {
			expect(new Hangul("\u1112\u1161\u11AB").josa("을/를")).toBe(
				"\u1112\u1161\u11AB을",
			);
			expect(() => new Hangul("사과").josa("을")).toThrow();
		});

		it("should select additional pairs and return the particle alone", () => {
			expect(new Hangul("사과").josa("아/야")).toBe("사과야");
			expect(new Hangul("수박").josa("아/야")).toBe("수박아");
			expect(new Hangul("사과").josa("이라고/라고")).toBe("사과라고");
			expect(new Hangul("수박").josaParticle("을/를")).toBe("을");
			expect(new Hangul("사과").josaParticle("를/을")).toBe("를");
			expect(() => new Hangul("사과").josaParticle("을")).toThrow();
		});
	});

	describe("units and grouped disassembly", () => {
		it("should expose syllable units", () => {
			const hangul = new Hangul("가A값");

			expect(hangul.length).toBe(3);
			expect(hangul.get(0)?.original).toBe("가");
			expect(hangul.get(0)?.isHangul).toBe(true);
			expect(hangul.get(0)?.choseong).toBe("ㄱ");
			expect(hangul.get(0)?.jungseong).toBe("ㅏ");
			expect(hangul.get(0)?.jongseong).toBeNull();
			expect(hangul.get(1)?.original).toBe("A");
			expect(hangul.get(1)?.isHangul).toBe(false);
			expect(hangul.get(2)?.jongseong).toBe("ㅄ");
			expect(hangul.get(3)).toBeNull();
		});

		it("should disassemble into groups", () => {
			expect(new Hangul("안녕").disassembleToGroups()).toEqual([
				["ㅇ", "ㅏ", "ㄴ"],
				["ㄴ", "ㅕ", "ㅇ"],
			]);
			expect(new Hangul("값").disassembleToGroups()).toEqual([
				["ㄱ", "ㅏ", "ㅂ", "ㅅ"],
			]);
			expect(new Hangul("가A!").disassembleToGroups()).toEqual([
				["ㄱ", "ㅏ"],
				["A"],
				["!"],
			]);
		});
	});

	describe("assemble function", () => {
		it("should compose basic and compound syllables", () => {
			expect(assemble("ㄱㅏ")).toBe("가");
			expect(assemble("ㄱㅘ")).toBe("과");
			expect(assemble("ㄱㅏㅂㅅ")).toBe("값");
		});

		it("should preserve syllable boundaries and non-Jamo text", () => {
			expect(assemble("ㅇㅏㄴㄴㅕㅇ")).toBe("안녕");
			expect(assemble("ㄱㅏㄱㅅㅏ")).toBe("각사");
			expect(assemble("Hello ㄱㅏ!")).toBe("Hello 가!");
			expect(assemble("ㄱ")).toBe("ㄱ");
		});

		it("should allow callers to choose the ambiguity policy", () => {
			expect(assemble("ㄱㅏㄱㅅㅏ")).toBe(
				assemble("ㄱㅏㄱㅅㅏ", "next-syllable"),
			);
			expect(assemble("ㄱㅏㄱㅅㅏ", "next-syllable")).toBe("각사");
			expect(assemble("ㄱㅏㄱㅅㅏ", "compound-final")).toBe("갃ㅏ");
			expect(() =>
				assemble("ㄱㅏ", "unknown" as "next-syllable"),
			).toThrow();
		});

		it("should treat precomposed compound finals as unambiguous", () => {
			const compounds = [
				["ㄳ", "갃"],
				["ㄵ", "갅"],
				["ㄶ", "갆"],
				["ㄺ", "갉"],
				["ㄻ", "갊"],
				["ㄼ", "갋"],
				["ㄽ", "갌"],
				["ㄾ", "갍"],
				["ㄿ", "갎"],
				["ㅀ", "갏"],
				["ㅄ", "값"],
			] as const;

			for (const [compound, syllable] of compounds) {
				const input = `ㄱㅏ${compound}ㅏ`;
				const expected = `${syllable}ㅏ`;

				expect(assemble(input)).toBe(expected);
				expect(assemble(input, "next-syllable")).toBe(expected);
				expect(assemble(input, "compound-final")).toBe(expected);
			}
		});

		it("should leave incomplete and unsupported Jamo unchanged", () => {
			expect(assemble("")).toBe("");
			expect(assemble("ㄱ")).toBe("ㄱ");
			expect(assemble("ㄸㅃㅉ")).toBe("ㄸㅃㅉ");
			expect(assemble("가")).toBe("가");
		});
	});
});

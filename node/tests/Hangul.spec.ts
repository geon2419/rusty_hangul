import { describe, it, expect } from "vitest";
import { Hangul } from "../index";

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

		// NFD 입력을 그대로 유지하는지 테스트
		it("should pass through NFD input", () => {
			const nfd = "\u1100\u1161\u11AB";
			expect(new Hangul(nfd).disassemble()).toBe(nfd);
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

		// NFD 입력을 그대로 유지하는지 테스트
		it("should pass through NFD input", () => {
			const nfd = "\u1100\u1161\u11AB";
			expect(new Hangul(nfd).getChoseong()).toBe(nfd);
		});

		// 동일 인스턴스에서 반복 호출 시 결과가 동일한지 테스트
		it("should return the same result on repeated calls", () => {
			const hangul = new Hangul("안녕 Hello");
			expect(hangul.getChoseong()).toBe("ㅇㄴ Hello");
			expect(hangul.getChoseong()).toBe("ㅇㄴ Hello");
		});
	});
});

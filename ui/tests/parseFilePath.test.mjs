import assert from "node:assert/strict";
import test from "node:test";

import { isAbsDiskPath, normalizePathSeparators, parseFilePath } from "../src/parseFilePath.ts";

test("normalizePathSeparators converts Windows separators", () => {
  assert.equal(
    normalizePathSeparators("context_fork\\simulator.py"),
    "context_fork/simulator.py",
  );
});

test("isAbsDiskPath recognizes Windows drive paths", () => {
  assert.equal(isAbsDiskPath("C:\\Users\\me\\repo\\file.py"), true);
  assert.equal(isAbsDiskPath("context_fork/simulator.py"), false);
});

test("parseFilePath strips a Windows repo prefix to a repo-relative path", () => {
  const repo = "C:\\Users\\user\\AppData\\Local\\openresearch\\repos\\owner\\repo";
  const file = "C:\\Users\\user\\AppData\\Local\\openresearch\\repos\\owner\\repo\\context_fork\\simulator.py";
  assert.deepEqual(parseFilePath(file, repo, "session-1"), {
    path: "context_fork/simulator.py",
    sessionId: "session-1",
  });
});

test("parseFilePath matches openresearch hub layout on Windows", () => {
  const file =
    "C:/Users/user/AppData/Local/openresearch/repos/owner/repo/context_fork/simulator.py";
  assert.deepEqual(parseFilePath(file), {
    path: "context_fork/simulator.py",
    sessionId: undefined,
  });
});

test("parseFilePath keeps relative paths in the click context", () => {
  assert.deepEqual(parseFilePath("context_fork/simulator.py", undefined, "session-1"), {
    path: "context_fork/simulator.py",
    sessionId: "session-1",
  });
});

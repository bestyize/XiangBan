import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';

const context = vm.createContext({});
vm.runInContext(fs.readFileSync(new URL('../public/game-packs/merge-2048/rules.js', import.meta.url), 'utf8'), context);
const rules = context.Xiaban2048;
const plain = value => JSON.parse(JSON.stringify(value));
const rowBoard = row => [row, [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];

test('a tile merges once; motion paths retain both sources and skip empty cells', () => {
  const board = rowBoard([2, 2, 2, 2]);
  const snapshot = structuredClone(board);
  const result = plain(rules.move(board, 'left'));
  assert.deepEqual(result.board, rowBoard([4, 4, 0, 0]));
  assert.equal(result.score, 8);
  assert.deepEqual(result.motions.map(m => [m.from.col, m.to.col]), [[0, 0], [1, 0], [2, 1], [3, 1]]);
  assert.deepEqual(board, snapshot);
  assert.deepEqual(plain(rules.move(rowBoard([2, 0, 2, 4]), 'left')).board, rowBoard([4, 4, 0, 0]));
});

test('all directions preserve tile mass and animate every source to its result cell', () => {
  const board = [[2, 0, 2, 4], [4, 4, 8, 0], [0, 8, 8, 16], [32, 0, 16, 16]];
  for (const direction of ['left', 'right', 'up', 'down']) {
    const result = plain(rules.move(board, direction));
    const reconstructed = rules.createBoard();
    for (const { from, to } of result.motions) reconstructed[to.row][to.col] += board[from.row][from.col];
    assert.deepEqual(plain(reconstructed), result.board);
    assert.equal(result.motions.length, board.flat().filter(Boolean).length);
    assert.equal(result.board.flat().reduce((a, b) => a + b), board.flat().reduce((a, b) => a + b));
  }
  assert.deepEqual(plain(rules.move(rowBoard([2, 2, 4, 0]), 'right')).board, rowBoard([0, 0, 4, 4]));
});

test('invalid moves, spawn probabilities, winning and terminal boards retain their rules', () => {
  assert.equal(rules.move(rowBoard([2, 0, 0, 0]), 'left').moved, false);
  assert.equal(rules.addTile(rules.createBoard(), () => .89).board.flat().find(Boolean), 2);
  assert.equal(rules.addTile(rules.createBoard(), () => .9).board.flat().find(Boolean), 4);
  assert.equal(rules.hasWon(rules.move(rowBoard([1024, 1024, 0, 0]), 'left').board), true);
  const lost = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
  assert.equal(rules.canMove(lost), false);
  assert.equal(rules.addTile(lost).cell, null);
  assert.throws(() => rules.move(lost, 'diagonal'));
});

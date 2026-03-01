import test from 'node:test';
import assert from 'node:assert/strict';
import { StateMachine } from '../../src/control/state-machine.js';

test('allows expected transition', () => {
  const fsm = new StateMachine();
  assert.equal(fsm.canTransition('PENDING', 'PLANNING'), true);
  assert.equal(fsm.canTransition('PENDING', 'DONE'), false);
});

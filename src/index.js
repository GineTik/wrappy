#!/usr/bin/env node

import { ContainerManager } from './core/ContainerManager.js';
import { LocalRepository } from './repository/LocalRepository.js';
import { CLI } from './cli/CLI.js';

async function main() {
  const repository = new LocalRepository();
  const containerManager = new ContainerManager(repository);
  const cli = new CLI(containerManager);

  await cli.start();
}

main().catch(console.error);
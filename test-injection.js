#!/usr/bin/env node

import { DependencyInjector } from './src/core/DependencyInjector.js';
import path from 'path';

async function testDependencyInjection() {
  console.log('🧪 Тестування Dependency Injection');
  console.log('====================================');
  
  const containerPath = './examples/hello-world';
  const packagesPath = './examples/packages';
  const dependencies = ['node-18@18.17.0'];
  
  try {
    // Спочатку очистимо попередні injection'и
    await DependencyInjector.removeDependencyInjection(containerPath);
    
    // Виконаємо injection
    console.log('📦 Інжектуємо залежності...');
    await DependencyInjector.injectDependencies(containerPath, dependencies, packagesPath);
    
    // Перевіримо створені команди
    console.log('\n📋 Список інжектованих команд:');
    const commands = await DependencyInjector.listInjectedCommands(containerPath);
    console.log(commands.length > 0 ? commands : 'Немає команд');
    
    console.log('\n✅ Dependency injection завершено!');
    console.log('Тепер можете запустити: cd examples/hello-world && ./scripts/default.sh');
    
  } catch (error) {
    console.error('❌ Помилка:', error.message);
  }
}

testDependencyInjection();
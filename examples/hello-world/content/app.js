const message = process.env.MESSAGE || 'Привіт, світ!';

console.log('╔══════════════════════════════════════╗');
console.log('║          HELLO WORLD APP             ║');
console.log('╚══════════════════════════════════════╝');
console.log();
console.log(`📢 ${message}`);
console.log();
console.log('🚀 Контейнер працює успішно!');
console.log('⏰ Час:', new Date().toLocaleString());
console.log('📍 Робоча директорія:', process.cwd());
console.log();

setTimeout(() => {
  console.log('✅ Програма завершена');
}, 1000);
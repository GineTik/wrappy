/**
 * @typedef {Object} ContainerManifest
 * @property {string} name - Назва контейнера
 * @property {string} version - Версія контейнера
 * @property {string} description - Опис контейнера
 * @property {string} author - Автор контейнера
 * @property {string[]} tags - Теги для категоризації
 * @property {Object} scripts - Скрипти запуску
 * @property {string} scripts.default - Дефолтний скрипт
 * @property {Object<string, string>} scripts.named - Іменовані сценарії
 * @property {string[]} dependencies - Залежності (пакети)
 * @property {Object} permissions - Дозволи контейнера
 * @property {string[]} permissions.api - Доступні API
 * @property {string[]} permissions.resources - Доступ до ресурсів
 * @property {Object} environment - Змінні середовища
 */

/**
 * @typedef {Object} Container
 * @property {string} id - Унікальний ідентифікатор
 * @property {ContainerManifest} manifest - Маніфест контейнера
 * @property {string} path - Шлях до контейнера
 * @property {string} status - Статус: 'installed' | 'running' | 'stopped'
 * @property {Date} createdAt - Дата створення
 * @property {Date} updatedAt - Дата оновлення
 */

/**
 * @typedef {Object} Package
 * @property {string} id - Унікальний ідентифікатор
 * @property {string} name - Назва пакета
 * @property {string} version - Версія пакета
 * @property {string} path - Шлях до пакета
 * @property {string[]} usedBy - Контейнери, що використовують цей пакет
 */

export const ContainerStatus = {
  INSTALLED: 'installed',
  RUNNING: 'running',
  STOPPED: 'stopped'
};

export const PackageType = {
  LIBRARY: 'library',
  SERVICE: 'service',
  RUNTIME: 'runtime'
};
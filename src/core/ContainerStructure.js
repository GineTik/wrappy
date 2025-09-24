import fs from 'fs/promises';
import path from 'path';
import { ContainerValidator } from './ContainerValidator.js';

export class ContainerStructure {
  
  static async createContainerStructure(basePath, manifest) {
    this._validateManifestForCreation(manifest);
    
    const containerPath = path.join(basePath, manifest.name);
    
    await this._createDirectoryStructure(containerPath);
    await this._createManifestFile(containerPath, manifest);
    await this._createDefaultScript(containerPath);
    await this._createConfigFiles(containerPath);
    
    return containerPath;
  }
  
  static _validateManifestForCreation(manifest) {
    const requiredFields = ['name', 'version', 'description'];
    
    for (const field of requiredFields) {
      if (!manifest[field] || typeof manifest[field] !== 'string') {
        throw new Error(`Manifest field "${field}" is required and must be a string`);
      }
    }
  }
  
  static async _createDirectoryStructure(containerPath) {
    const directories = [
      containerPath,
      path.join(containerPath, 'scripts'),
      path.join(containerPath, 'content'),
      path.join(containerPath, 'config')
    ];
    
    for (const dir of directories) {
      await fs.mkdir(dir, { recursive: true });
    }
  }
  
  static async _createManifestFile(containerPath, manifest) {
    const defaultManifest = {
      name: manifest.name,
      version: manifest.version,
      description: manifest.description,
      author: manifest.author || '',
      tags: manifest.tags || [],
      scripts: {
        default: './scripts/default.sh',
        ...manifest.scripts
      },
      dependencies: manifest.dependencies || [],
      permissions: {
        api: [],
        resources: [],
        ...manifest.permissions
      },
      environment: manifest.environment || {}
    };
    
    const manifestPath = path.join(containerPath, 'manifest.json');
    await fs.writeFile(manifestPath, JSON.stringify(defaultManifest, null, 2));
  }
  
  static async _createDefaultScript(containerPath) {
    const defaultScript = `#!/bin/bash

# Дефолтний скрипт запуску контейнера
echo "Запуск контейнера..."

# Тут буде логіка запуску вашого додатку
# Наприклад:
# cd content && npm start
# або 
# ./content/app

echo "Контейнер запущено"
`;
    
    const scriptPath = path.join(containerPath, 'scripts', 'default.sh');
    await fs.writeFile(scriptPath, defaultScript);
    await fs.chmod(scriptPath, 0o755);
  }
  
  static async _createConfigFiles(containerPath) {
    const permissionsConfig = {
      api: [],
      resources: [],
      network: false,
      filesystem: {
        read: [],
        write: []
      }
    };
    
    const environmentConfig = {
      variables: {},
      path: [],
      workingDirectory: "./content"
    };
    
    const configPath = path.join(containerPath, 'config');
    
    await fs.writeFile(
      path.join(configPath, 'permissions.json'),
      JSON.stringify(permissionsConfig, null, 2)
    );
    
    await fs.writeFile(
      path.join(configPath, 'environment.json'),
      JSON.stringify(environmentConfig, null, 2)
    );
  }

  static async loadContainer(containerPath) {
    await ContainerValidator.validateContainerStructure(containerPath);
    
    const manifest = await this._loadManifest(containerPath);
    const permissions = await this._loadPermissions(containerPath);
    const environment = await this._loadEnvironment(containerPath);
    
    return {
      path: containerPath,
      manifest,
      permissions,
      environment
    };
  }
  
  static async _loadManifest(containerPath) {
    const manifestPath = path.join(containerPath, 'manifest.json');
    const content = await fs.readFile(manifestPath, 'utf-8');
    return JSON.parse(content);
  }
  
  static async _loadPermissions(containerPath) {
    const permissionsPath = path.join(containerPath, 'config', 'permissions.json');
    try {
      const content = await fs.readFile(permissionsPath, 'utf-8');
      return JSON.parse(content);
    } catch {
      return { api: [], resources: [], network: false, filesystem: { read: [], write: [] } };
    }
  }
  
  static async _loadEnvironment(containerPath) {
    const environmentPath = path.join(containerPath, 'config', 'environment.json');
    try {
      const content = await fs.readFile(environmentPath, 'utf-8');
      return JSON.parse(content);
    } catch {
      return { variables: {}, path: [], workingDirectory: "./content" };
    }
  }
}
import fs from 'fs/promises';
import path from 'path';

export class ContainerValidator {
  
  static async validateContainerStructure(containerPath) {
    const errors = [];
    
    await this._validateManifest(containerPath, errors);
    await this._validateScriptsDirectory(containerPath, errors);
    await this._validateContentDirectory(containerPath, errors);
    await this._validateConfigDirectory(containerPath, errors);
    
    if (errors.length > 0) {
      throw new Error(`Container validation failed:\n${errors.join('\n')}`);
    }
  }

  static async _validateManifest(containerPath, errors) {
    const manifestPath = path.join(containerPath, 'manifest.json');
    
    try {
      const manifestContent = await fs.readFile(manifestPath, 'utf-8');
      const manifest = JSON.parse(manifestContent);
      
      this._validateManifestFields(manifest, errors);
    } catch (error) {
      if (error.code === 'ENOENT') {
        errors.push('manifest.json is required');
      } else if (error instanceof SyntaxError) {
        errors.push('manifest.json contains invalid JSON');
      } else {
        errors.push(`Failed to read manifest.json: ${error.message}`);
      }
    }
  }
  
  static _validateManifestFields(manifest, errors) {
    const requiredFields = ['name', 'version', 'description'];
    
    for (const field of requiredFields) {
      if (!manifest[field] || typeof manifest[field] !== 'string') {
        errors.push(`manifest.json: "${field}" is required and must be a string`);
      }
    }
    
    if (manifest.scripts && typeof manifest.scripts !== 'object') {
      errors.push('manifest.json: "scripts" must be an object');
    }
    
    if (manifest.dependencies && !Array.isArray(manifest.dependencies)) {
      errors.push('manifest.json: "dependencies" must be an array');
    }
    
    if (manifest.permissions && typeof manifest.permissions !== 'object') {
      errors.push('manifest.json: "permissions" must be an object');
    }
  }

  static async _validateScriptsDirectory(containerPath, errors) {
    const scriptsPath = path.join(containerPath, 'scripts');
    
    try {
      const stats = await fs.stat(scriptsPath);
      if (!stats.isDirectory()) {
        errors.push('scripts/ must be a directory');
        return;
      }
      
      const defaultScriptPath = path.join(scriptsPath, 'default.sh');
      try {
        await fs.access(defaultScriptPath);
      } catch {
        errors.push('scripts/default.sh is required');
      }
      
    } catch (error) {
      if (error.code === 'ENOENT') {
        errors.push('scripts/ directory is required');
      }
    }
  }

  static async _validateContentDirectory(containerPath, errors) {
    const contentPath = path.join(containerPath, 'content');
    
    try {
      const stats = await fs.stat(contentPath);
      if (!stats.isDirectory()) {
        errors.push('content/ must be a directory');
      }
    } catch (error) {
      if (error.code === 'ENOENT') {
        errors.push('content/ directory is required');
      }
    }
  }

  static async _validateConfigDirectory(containerPath, errors) {
    const configPath = path.join(containerPath, 'config');
    
    try {
      const stats = await fs.stat(configPath);
      if (!stats.isDirectory()) {
        errors.push('config/ must be a directory');
      }
    } catch (error) {
    }
  }
}